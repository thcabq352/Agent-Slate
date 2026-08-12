use slate_domain::{apply_ad_actions, new_project, AdAction};

#[test]
fn create_scene_and_shot_emits_receipts() {
    let mut p = new_project("T");
    let actions = vec![
        AdAction::CreateScene {
            name: "Rooftop".into(),
            synopsis: Some("Chase".into()),
        },
        AdAction::CreateShot {
            scene: "Rooftop".into(),
            name: Some("Shot 01".into()),
            intent: Some("Establish".into()),
            prompt: Some("# Subject\nKaia runs\n".into()),
            spec: None,
            target_model: None,
            max_chars: None,
            beat_sheet: None,
        },
    ];
    let r = apply_ad_actions(&mut p, &actions);
    assert_eq!(p.scenes.len(), 1);
    assert_eq!(p.scenes[0].shots.len(), 1);
    assert!(r.receipts.iter().any(|x| x.contains("Created scene")));
    assert!(r.receipts.iter().any(|x| x.contains("Created")));
    assert!(r.focus_shot_id.is_some());
}

#[test]
fn duplicate_scene_is_skipped() {
    let mut p = new_project("T");
    let a = AdAction::CreateScene {
        name: "A".into(),
        synopsis: None,
    };
    apply_ad_actions(&mut p, &[a.clone()]);
    let r = apply_ad_actions(&mut p, &[a]);
    assert_eq!(p.scenes.len(), 1);
    assert!(r.receipts.iter().any(|x| x.contains("already exists")));
}
