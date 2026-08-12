use slate_domain::{new_project, Project};

#[test]
fn new_project_roundtrips_json() {
    let p = new_project("Night Market");
    let json = serde_json::to_string_pretty(&p).unwrap();
    let back: Project = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "Night Market");
    assert!(back.scenes.is_empty());
    assert_eq!(back.defaults.target_model, "seedance-2");
    assert_eq!(back.defaults.brain, slate_domain::BrainBackend::Claude);
}
