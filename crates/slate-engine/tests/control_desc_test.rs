#[test]
fn descriptor_path_ends_with_control_json() {
    let p = slate_engine::control_desc::descriptor_path();
    assert!(p.ends_with("control.json"));
}
