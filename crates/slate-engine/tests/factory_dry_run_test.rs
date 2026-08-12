#[test]
fn parse_scene_brief_json() {
    let v = serde_json::json!({
        "title": "Chase",
        "logline": "x",
        "world": "y",
        "shot_count": 6,
        "duration_sec": 8,
        "aspect_ratio": "16:9",
        "pack_id": "default-still",
        "characters": [{"name": "Kaia", "one_liner": "courier"}],
        "location": {"name": "Rooftops", "description": "wet neon"},
        "style_notes": "cinematic"
    });
    let b: slate_engine::factory::SceneBrief = serde_json::from_value(v).unwrap();
    assert_eq!(b.shot_count, 6);
}

#[tokio::test]
async fn dry_run_factory_creates_project_and_takes() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("SLATE_DATA_DIR", dir.path());
    std::env::set_var("SLATE_DRY_RUN", "1");
    let ctx = slate_engine::EngineCtx::for_test(dir.path()).await;
    let res = slate_engine::factory::run_film_factory(
        &ctx,
        slate_engine::factory::FilmFactoryArgs {
            brief: "Rainy neon rooftop chase".into(),
            pack_id: Some("default-still".into()),
            brain: None,
            shot_count: Some(4),
            project_name: Some("Test".into()),
        },
    )
    .await;
    assert!(res.ok, "{:?}", res.warnings);
    assert_eq!(res.shots.len(), 4);
    assert!(res.shots.iter().all(|s| s.take_path.is_some()));
    std::env::remove_var("SLATE_DATA_DIR");
    std::env::remove_var("SLATE_DRY_RUN");
}
