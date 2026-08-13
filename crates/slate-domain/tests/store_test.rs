use slate_domain::{
    apply_ad_actions, circle_take, create_project, list_projects, open_project, save_project,
    AdAction, Take, TakeRating,
};
use std::env;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_data_dir<R>(f: impl FnOnce() -> R) -> R {
    let _g = ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    env::set_var("SLATE_DATA_DIR", dir.path());
    let out = f();
    env::remove_var("SLATE_DATA_DIR");
    out
}

#[test]
fn create_open_list_roundtrip() {
    with_data_dir(|| {
        let p = create_project("Alpha").unwrap();
        let loaded = open_project(&p.id).unwrap().expect("exists");
        assert_eq!(loaded.name, "Alpha");
        let metas = list_projects().unwrap();
        assert_eq!(metas.len(), 1);
        assert_eq!(metas[0].shot_count, 0);
    });
}

#[test]
fn circle_take_marks_latest_circled() {
    with_data_dir(|| {
        let mut p = create_project("Takes").unwrap();
        apply_ad_actions(
            &mut p,
            &[
                AdAction::CreateScene {
                    name: "Rooftop".into(),
                    synopsis: None,
                },
                AdAction::CreateShot {
                    scene: "Rooftop".into(),
                    name: Some("Wide".into()),
                    intent: None,
                    prompt: None,
                    spec: None,
                    target_model: None,
                    max_chars: None,
                    beat_sheet: None,
                },
            ],
        );
        let shot = &mut p.scenes[0].shots[0];
        shot.takes.push(Take {
            id: "take-old".into(),
            logged_at: "2026-08-01T00:00:00.000Z".into(),
            model: "default-still".into(),
            prompt: "a".into(),
            rating: TakeRating::Good,
            notes: String::new(),
            media_path: Some("/tmp/old.png".into()),
        });
        shot.takes.push(Take {
            id: "take-new".into(),
            logged_at: "2026-08-13T00:00:00.000Z".into(),
            model: "default-still".into(),
            prompt: "b".into(),
            rating: TakeRating::NoGood,
            notes: "path | quality".into(),
            media_path: Some("/tmp/new.png".into()),
        });
        save_project(&mut p).unwrap();

        let out = circle_take(&p.id, None, None).unwrap();
        assert_eq!(out.take_id, "take-new");
        assert_eq!(out.rating, TakeRating::Circled);
        let loaded = open_project(&p.id).unwrap().unwrap();
        let take = &loaded.scenes[0].shots[0].takes[1];
        assert_eq!(take.rating, TakeRating::Circled);
        assert!(take.notes.contains("human approved"));
    });
}
