#[test]
fn descriptor_path_is_engine_control_json() {
    let p = slate_engine::control_desc::descriptor_path();
    assert_eq!(
        p.file_name().and_then(|n| n.to_str()),
        Some(slate_engine::control_desc::DESCRIPTOR_FILE)
    );
}

#[test]
fn app_name_is_slate_engine() {
    assert_eq!(slate_engine::control_desc::APP_NAME, "slate-engine");
}
