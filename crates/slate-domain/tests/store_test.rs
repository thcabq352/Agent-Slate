use slate_domain::{create_project, list_projects, open_project, save_project};
use std::env;

#[test]
fn create_open_list_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    env::set_var("SLATE_DATA_DIR", dir.path());
    let p = create_project("Alpha").unwrap();
    let loaded = open_project(&p.id).unwrap().expect("exists");
    assert_eq!(loaded.name, "Alpha");
    let metas = list_projects().unwrap();
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].shot_count, 0);
    env::remove_var("SLATE_DATA_DIR");
}
