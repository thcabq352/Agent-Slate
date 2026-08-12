//! Live Comfy LTX video smoke — only when SLATE_LIVE_VIDEO=1 and Comfy is on :8188.
//! Ignored by default; 49-frame distilled T2V is minutes on a 16 GB card.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::json;
use slate_comfy::{generate_to_file, ComfyClient};

#[tokio::test]
#[ignore]
async fn live_default_video_pack_generates_mp4() {
    if std::env::var("SLATE_LIVE_VIDEO").ok().as_deref() != Some("1") {
        eprintln!("skip: set SLATE_LIVE_VIDEO=1 and run with --ignored");
        return;
    }
    std::env::remove_var("SLATE_DRY_RUN");

    let client = ComfyClient::default_local().expect("build client");
    client
        .health()
        .await
        .expect("Comfy health — start API on http://127.0.0.1:8188");

    let packs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../workflows/packs");
    let dest = std::env::temp_dir().join(format!("slate-live-video-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dest);

    let mut values = HashMap::new();
    values.insert(
        "positive".into(),
        json!("cinematic commercial shot, soft rain on a neon rooftop, smooth camera, photoreal"),
    );
    values.insert(
        "negative".into(),
        json!("blurry, low quality, watermark, text overlay, jitter"),
    );
    values.insert("width".into(), json!(768));
    values.insert("height".into(), json!(432));
    values.insert("frames".into(), json!(49));

    let path = tokio::time::timeout(
        Duration::from_secs(600),
        generate_to_file(&client, &packs, "default-video", &values, &dest),
    )
    .await
    .expect("timed out waiting for Comfy LTX")
    .expect("generate_to_file");

    assert!(path.exists(), "output missing: {}", path.display());
    let meta = std::fs::metadata(&path).unwrap();
    assert!(meta.len() > 20_000, "file too small ({} bytes)", meta.len());
    let name = path.file_name().unwrap().to_string_lossy();
    assert!(
        name.ends_with(".mp4") || name.ends_with(".webm") || name.ends_with(".mkv"),
        "expected video file, got {name}"
    );
    eprintln!("live video ok: {} ({} bytes)", path.display(), meta.len());
}
