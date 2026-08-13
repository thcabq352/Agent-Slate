use slate_domain::{new_project, Project};

#[test]
fn new_project_roundtrips_json() {
    let p = new_project("Night Market");
    let json = serde_json::to_string_pretty(&p).unwrap();
    let back: Project = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "Night Market");
    assert!(back.scenes.is_empty());
    assert_eq!(back.defaults.target_model, "seedance-2");
    assert_eq!(back.defaults.brain, slate_domain::BrainBackend::Cursor);
}

#[test]
fn claude_alias_deserializes_as_cursor() {
    let raw = r#""claude""#;
    let back: slate_domain::BrainBackend = serde_json::from_str(raw).unwrap();
    assert_eq!(back, slate_domain::BrainBackend::Cursor);
    assert_eq!(serde_json::to_string(&back).unwrap(), r#""cursor""#);
}

#[test]
fn grok_backends_roundtrip() {
    let g45: slate_domain::BrainBackend = serde_json::from_str(r#""grok-4.5""#).unwrap();
    let g46: slate_domain::BrainBackend = serde_json::from_str(r#""grok-4.6""#).unwrap();
    assert_eq!(g45, slate_domain::BrainBackend::Grok45);
    assert_eq!(g46, slate_domain::BrainBackend::Grok46);
    assert_eq!(serde_json::to_string(&g45).unwrap(), r#""grok-4.5""#);
    assert_eq!(serde_json::to_string(&g46).unwrap(), r#""grok-4.6""#);
}

#[test]
fn old_voice_sheet_json_loads_without_grok_tts_fields() {
    let raw = r#"{
        "id": "voice-1",
        "name": "Marlow",
        "characterId": null,
        "ageGender": "61",
        "accent": "Chicago",
        "timbre": "dry",
        "pitch": "low",
        "pacing": "unhurried",
        "energy": "tired",
        "texture": "gravel",
        "emotionalRange": "narrow",
        "sampleLine": "Leave it.",
        "notes": ""
    }"#;
    let v: slate_domain::VoiceSheet = serde_json::from_str(raw).unwrap();
    assert_eq!(v.name, "Marlow");
    assert!(v.grok_voice_id.is_none());
    assert!(v.vo_path.is_none());
}
