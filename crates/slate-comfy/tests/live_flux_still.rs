//! Live Comfy still smoke — only runs when SLATE_LIVE_COMFY=1 and Comfy is on :8188.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::json;
use slate_comfy::{generate_to_file, ComfyClient};

#[tokio::test]
async fn live_default_still_pack_generates_png() {
    if std::env::var("SLATE_LIVE_COMFY").ok().as_deref() != Some("1") {
        eprintln!("skip: set SLATE_LIVE_COMFY=1 to run live Comfy still test");
        return;
    }
    // Ensure dry-run is off for this process
    std::env::remove_var("SLATE_DRY_RUN");

    let client = ComfyClient::default_local().expect("build client");
    client
        .health()
        .await
        .expect("Comfy health — start API on http://127.0.0.1:8188");

    let packs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../workflows/packs");
    let dest = std::env::temp_dir().join(format!("slate-live-still-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dest);

    let mut values = HashMap::new();
    values.insert(
        "positive".into(),
        json!("cinematic still of a courier on a rainy neon rooftop at night, photoreal"),
    );
    values.insert("negative".into(), json!("blurry, low quality, watermark"));
    values.insert("width".into(), json!(1024));
    values.insert("height".into(), json!(576));

    let path = tokio::time::timeout(
        Duration::from_secs(300),
        generate_to_file(&client, &packs, "default-still", &values, &dest),
    )
    .await
    .expect("timed out waiting for Comfy")
    .expect("generate_to_file");

    assert!(path.exists(), "output missing: {}", path.display());
    let meta = std::fs::metadata(&path).unwrap();
    assert!(meta.len() > 10_000, "file too small ({} bytes)", meta.len());
    eprintln!("live still ok: {} ({} bytes)", path.display(), meta.len());
}
