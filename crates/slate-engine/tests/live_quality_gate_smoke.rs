//! Live smoke: Comfy still + Ollama VL quality gate.
//! Run: `$env:SLATE_LIVE_GATE=1; cargo test -p slate-engine --test live_quality_gate_smoke -- --nocapture --test-threads=1`

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::json;
use slate_comfy::{generate_to_file, ComfyClient};
use slate_engine::config::load_config;
use slate_engine::quality_gate::judge_media;

#[tokio::test]
async fn live_still_then_quality_gate() {
    if std::env::var("SLATE_LIVE_GATE").ok().as_deref() != Some("1") {
        eprintln!("skip: set SLATE_LIVE_GATE=1 for live Comfy+Ollama gate smoke");
        return;
    }
    std::env::remove_var("SLATE_DRY_RUN");

    let client = ComfyClient::default_local().expect("client");
    client.health().await.expect("Comfy on :8188");

    let packs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../workflows/packs");
    let dest = std::env::temp_dir().join(format!("slate-gate-smoke-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dest);

    let prompt = "cinematic still of a courier on a rainy neon rooftop at night, photoreal, wet streets, cyan rim light";
    let mut values = HashMap::new();
    values.insert("positive".into(), json!(prompt));
    values.insert("negative".into(), json!("blurry, low quality, watermark, text"));
    values.insert("width".into(), json!(1024));
    values.insert("height".into(), json!(576));
    values.insert("seed".into(), json!(42u64));

    eprintln!("=== generate still (Flux pack) ===");
    let path = tokio::time::timeout(
        Duration::from_secs(300),
        generate_to_file(&client, &packs, "default-still", &values, &dest),
    )
    .await
    .expect("generate timeout")
    .expect("generate_to_file");

    let bytes = std::fs::metadata(&path).unwrap().len();
    eprintln!("generated: {} ({} bytes)", path.display(), bytes);
    assert!(bytes > 10_000, "image too small");

    let mut config = load_config();
    config.dry_run = false;
    // Prefer qwen3.5:9b; load_config already defaults
    eprintln!(
        "=== judge with model={} endpoint={} threshold={} ===",
        config.judge_model, config.judge_endpoint, config.judge_pass_threshold
    );

    let continuity = "Night city chase; protagonist in amber leather jacket; wet neon streets.";
    let gate = tokio::time::timeout(
        Duration::from_secs(300),
        judge_media(&config, &path, prompt, continuity),
    )
    .await
    .expect("judge timeout")
    .expect("judge_media");

    eprintln!("skipped={}", gate.skipped);
    if let Some(r) = &gate.skip_reason {
        eprintln!("skip_reason={r}");
    }
    let v = &gate.verdict;
    eprintln!(
        "accept={} overall={:.3} vq={:.3} cont={:.3} art={:.3} fid={:.3}",
        v.accept,
        v.overall,
        v.scores.visual_quality,
        v.scores.continuity,
        v.scores.artifacts,
        v.scores.prompt_fidelity
    );
    eprintln!("model={:?}", v.judge_model);
    eprintln!("summary={}", v.summary);
    eprintln!("issues={:?}", v.issues);
    eprintln!("retry_hints={:?}", v.retry_hints);

    assert!(!gate.skipped, "expected live VL judge, not skip: {:?}", gate.skip_reason);
    assert!(v.judge_model.is_some());
    assert!(
        (0.0..=1.0).contains(&v.overall),
        "overall out of range: {}",
        v.overall
    );
}
