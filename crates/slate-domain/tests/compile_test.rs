use slate_domain::{apply_ad_actions, compile_for_comfy, new_project, AdAction};

#[test]
fn compile_strips_headers_and_sets_size() {
    let mut p = new_project("T");
    apply_ad_actions(
        &mut p,
        &[
            AdAction::CreateScene {
                name: "S".into(),
                synopsis: None,
            },
            AdAction::CreateShot {
                scene: "S".into(),
                name: Some("01".into()),
                intent: None,
                prompt: Some("# Subject\nA red car\n\n# Mood\nTense\n".into()),
                spec: None,
                target_model: None,
                max_chars: None,
                beat_sheet: None,
            },
        ],
    );
    let shot = &p.scenes[0].shots[0];
    let c = compile_for_comfy(shot, "16:9");
    assert!(c.positive.contains("A red car"));
    assert!(!c.positive.contains("# Subject"));
    assert_eq!((c.width, c.height), (1280, 720));
    assert!(!c.negative.is_empty());
}
