//! Live 4-shot film factory: Comfy + Ollama quality gate.
//! `$env:SLATE_LIVE_FACTORY=1; cargo test -p slate-engine --test live_film_factory_smoke -- --nocapture --test-threads=1`

use std::time::Duration;

use slate_engine::config::load_config;
use slate_engine::factory::{run_film_factory, FilmFactoryArgs};
use slate_engine::tools::EngineCtx;

#[tokio::test]
async fn live_four_shot_factory() {
    if std::env::var("SLATE_LIVE_FACTORY").ok().as_deref() != Some("1") {
        eprintln!("skip: set SLATE_LIVE_FACTORY=1 for live 4-shot factory");
        return;
    }
    std::env::remove_var("SLATE_DRY_RUN");
    std::env::set_var("SLATE_JUDGE_MODEL", "qwen3.5:9b");

    let data = tempfile::tempdir().expect("tempdir");
    std::env::set_var("SLATE_DATA_DIR", data.path());

    let mut cfg = load_config();
    cfg.dry_run = false;
    cfg.data_dir = data.path().to_path_buf();
    cfg.brain_default = "local".into();
    cfg.judge_model = "qwen3.5:9b".into();
    // Prefer packs from workspace
    let packs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../workflows/packs");
    cfg.packs_dir = packs.canonicalize().unwrap_or(packs);

    let ctx = EngineCtx::new(cfg);

    let brief = "Night courier chase on rainy neon rooftops. \
        Protagonist Kaia, 29, cropped black hair, amber leather jacket. \
        Cinematic photoreal stills, wet streets, cyan rim light.";

    eprintln!("=== slate_film_factory live (4 shots) ===");
    eprintln!("data_dir={}", data.path().display());
    eprintln!("packs_dir={}", ctx.config.packs_dir.display());

    let result = tokio::time::timeout(
        Duration::from_secs(1800),
        run_film_factory(
            &ctx,
            FilmFactoryArgs {
                brief: brief.into(),
                pack_id: Some("default-still".into()),
                brain: Some(slate_domain::BrainBackend::Local),
                shot_count: Some(4),
                project_name: Some("Live Factory Smoke".into()),
            },
        ),
    )
    .await
    .expect("factory timed out (30 min)")
    ;

    eprintln!("ok={}", result.ok);
    eprintln!("project_id={}", result.project_id);
    eprintln!("scene_id={}", result.scene_id);
    eprintln!("elapsed_ms={}", result.elapsed_ms);
    eprintln!("--- receipts ---");
    for r in &result.receipts {
        eprintln!("{r}");
    }
    eprintln!("--- warnings ---");
    for w in &result.warnings {
        eprintln!("{w}");
    }
    eprintln!("--- shots ---");
    for s in &result.shots {
        eprintln!(
            "{} path={:?} err={:?} attempts={:?} quality={:?}",
            s.name,
            s.take_path.as_ref().map(|p| p.display().to_string()),
            s.error,
            s.attempts,
            s.quality.as_ref().map(|q| format!(
                "accept={} overall={:.2}",
                q.accept, q.overall
            ))
        );
    }

    assert!(
        result.ok,
        "factory not ok — warnings={:?} shots={:?}",
        result.warnings, result.shots
    );
    assert_eq!(result.shots.len(), 4, "expected 4 shots");
    let with_takes = result
        .shots
        .iter()
        .filter(|s| s.take_path.is_some())
        .count();
    assert!(
        with_takes >= 1,
        "expected at least one take file, got {with_takes}"
    );
}
