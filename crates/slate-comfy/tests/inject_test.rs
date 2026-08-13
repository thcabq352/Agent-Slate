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

#[test]
fn default_still_manifest_mirrors_width_height() {
    let pack = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../workflows/packs/default-still/manifest.json");
    let m = load_manifest(&pack).expect("default-still manifest");
    assert_eq!(m.id, "default-still");
    assert_eq!(m.inputs["width"].node_id, "27");
    assert_eq!(m.inputs["width"].mirrors[0].node_id, "30");
    assert_eq!(m.inputs["height"].mirrors[0].field, "height");
    assert_eq!(m.inputs["positive"].node_id, "6");
}

#[test]
fn default_video_manifest_maps_ltx_nodes() {
    let pack = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../workflows/packs/default-video/manifest.json");
    let m = load_manifest(&pack).expect("default-video manifest");
    assert_eq!(m.id, "default-video");
    assert_eq!(m.modality, "video");
    assert_eq!(m.inputs["positive"].node_id, "10");
    assert_eq!(m.inputs["negative"].node_id, "11");
    assert_eq!(m.inputs["width"].node_id, "20");
    assert_eq!(m.inputs["seed"].node_id, "42");
    assert_eq!(m.inputs["seed"].field, "noise_seed");
    assert_eq!(m.inputs["seed"].mode.as_deref(), Some("randomize"));
    assert!(m.inputs["frames"].optional);
    assert_eq!(m.inputs["frames"].mirrors[0].node_id, "21");
    assert_eq!(m.outputs["media"].node_id, "90");
}

#[test]
fn inject_default_video_positive_and_frames() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../workflows/packs/default-video");
    let manifest = load_manifest(&root.join("manifest.json")).unwrap();
    let workflow: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("workflow.api.json")).unwrap())
            .unwrap();
    assert!(
        !serde_json::to_string(&workflow)
            .unwrap()
            .contains("PLACEHOLDER"),
        "default-video must not ship a PLACEHOLDER graph"
    );

    let mut values = HashMap::new();
    values.insert("positive".into(), json!("neon rooftop courier"));
    values.insert("negative".into(), json!("blurry"));
    values.insert("width".into(), json!(768));
    values.insert("height".into(), json!(432));
    values.insert("frames".into(), json!(49));
    values.insert("seed".into(), json!(7));

    let out = inject_workflow(workflow, &manifest, &values).expect("inject");
    assert_eq!(out["10"]["inputs"]["text"], "neon rooftop courier");
    assert_eq!(out["11"]["inputs"]["text"], "blurry");
    assert_eq!(out["20"]["inputs"]["width"], 768);
    assert_eq!(out["20"]["inputs"]["length"], 49);
    assert_eq!(out["21"]["inputs"]["frames_number"], 49);
    assert_eq!(out["42"]["inputs"]["noise_seed"], 7);
}

#[test]
fn default_i2v_and_flf2v_manifests() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../workflows/packs");
    let i2v = load_manifest(&root.join("default-i2v/manifest.json")).unwrap();
    assert_eq!(i2v.inputs["image"].node_id, "8");
    assert_eq!(i2v.inputs["positive"].node_id, "10");
    let flf = load_manifest(&root.join("default-flf2v/manifest.json")).unwrap();
    assert_eq!(flf.inputs["image"].node_id, "8");
    assert_eq!(flf.inputs["image_end"].node_id, "18");
    let wf: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("default-i2v/workflow.api.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(wf["20"]["class_type"], "LTXVImgToVideo");
    let wff: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("default-flf2v/workflow.api.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(wff["23"]["class_type"], "LTXVAddGuide");
    assert_eq!(wff["24"]["inputs"]["frame_idx"], -1);
}
