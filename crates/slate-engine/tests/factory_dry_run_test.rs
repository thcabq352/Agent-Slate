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
