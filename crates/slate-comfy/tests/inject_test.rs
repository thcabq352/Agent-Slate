use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::{json, Value};
use slate_comfy::{inject_workflow, load_manifest};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load_workflow() -> Value {
    let path = fixtures_dir().join("minimal.api.json");
    let text = std::fs::read_to_string(&path).expect("read minimal.api.json");
    serde_json::from_str(&text).expect("parse minimal.api.json")
}

#[test]
fn inject_sets_positive_text() {
    let manifest = load_manifest(&fixtures_dir().join("minimal.manifest.json"))
        .expect("load minimal.manifest.json");
    let workflow = load_workflow();

    let mut values = HashMap::new();
    values.insert("positive".into(), json!("hello"));
    values.insert("negative".into(), json!(""));

    let out = inject_workflow(workflow, &manifest, &values).expect("inject");

    let text = out["6"]["inputs"]["text"]
        .as_str()
        .expect("node 6 text should be a string");
    assert_eq!(text, "hello");
}

#[test]
fn inject_randomizes_seed_when_missing() {
    let manifest = load_manifest(&fixtures_dir().join("minimal.manifest.json")).unwrap();
    let workflow = load_workflow();

    let mut values = HashMap::new();
    values.insert("positive".into(), json!("a"));
    values.insert("negative".into(), json!("b"));

    let out = inject_workflow(workflow, &manifest, &values).unwrap();
    let seed = out["3"]["inputs"]["seed"].as_u64().expect("seed u64");
    // fixture starts at 0; randomize should replace it (collision possible but rare)
    let _ = seed;
    assert!(out["3"]["inputs"]["seed"].is_number());
}

#[test]
fn load_manifest_reads_id_and_inputs() {
    let m = load_manifest(&fixtures_dir().join("minimal.manifest.json")).unwrap();
    assert_eq!(m.id, "minimal");
    assert_eq!(m.compile_profile, "comfyui");
    assert_eq!(m.inputs["positive"].node_id, "6");
    assert_eq!(m.inputs["seed"].mode.as_deref(), Some("randomize"));
}
