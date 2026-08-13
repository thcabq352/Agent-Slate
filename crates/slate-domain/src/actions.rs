//! First AD action contract and apply logic.
//! Port of `src/renderer/src/lib/firstAD.ts` `applyAdActions`.

use serde::{Deserialize, Serialize};

use crate::types::{
    ArtDeptKind, ArtDeptSheet, BeatDirection, BrainBackend, CharacterSheet, InteriorExterior,
    LocationSheet, MusicCue, Project, ProjectDefaults, ScenarioTab, Scene, Shot, ShotSpec,
    VocalsPreference, VoiceSheet,
};
use crate::uid::uid;

// ---- Spec / defaults patches ----

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpecPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_sec: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fps: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lens: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub movement: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DefaultsPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fps: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_sec: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brain: Option<BrainBackend>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_model: Option<String>,
}

// ---- AdAction ----

/// Actions the First AD may emit. Serde tag field is `"type"` with snake_case
/// variant names matching `firstAD.ts` (`create_scene`, `update_shot`, …).
/// Struct fields serialize as camelCase for JSON parity with the TS contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdAction {
    #[serde(rename_all = "camelCase")]
    UpdateProject {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        logline: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        world: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        defaults: Option<DefaultsPatch>,
    },
    #[serde(rename_all = "camelCase")]
    CreateScene {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        synopsis: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    UpdateScene {
        scene: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        synopsis: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    CreateShot {
        scene: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        intent: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prompt: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spec: Option<SpecPatch>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target_model: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_chars: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        beat_sheet: Option<Vec<BeatDirection>>,
    },
    #[serde(rename_all = "camelCase")]
    UpdateShot {
        shot: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        intent: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prompt: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spec: Option<SpecPatch>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target_model: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_chars: Option<Option<u32>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        beat_sheet: Option<Option<Vec<BeatDirection>>>,
    },
    #[serde(rename_all = "camelCase")]
    AddCharacter {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        age: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gender: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ethnicity: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        face_features: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hair: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        clothing: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expression: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        eye_direction: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mood: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        environment: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key_light_side: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lighting_mood: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    AddLocation {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        interior_exterior: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        time_of_day: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        weather: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        architecture: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        textures: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        practical_lights: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    AddArt {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        materials: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        condition: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        era: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        distinctive: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    AddMusicCue {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        scene_ref: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        intent: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        genre: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mood: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tempo: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        instrumentation: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        era: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        structure: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        vocals: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lyric_theme: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_sec: Option<f64>,
    },
    #[serde(rename_all = "camelCase")]
    AddVoice {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        character_name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        age_gender: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        accent: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timbre: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pitch: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pacing: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        energy: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        texture: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        emotional_range: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sample_line: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Select {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        scene: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shot: Option<String>,
    },
}

// ---- Apply result ----

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub receipts: Vec<String>,
    pub focus_scene_id: Option<String>,
    pub focus_shot_id: Option<String>,
}

// ---- Helpers ----

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn opt_str(v: &Option<String>) -> String {
    v.as_deref().unwrap_or("").to_string()
}

fn find_scene_idx(p: &Project, ref_: &str) -> Option<usize> {
    let needle = ref_.trim().to_lowercase();
    p.scenes.iter().position(|s| s.id == ref_).or_else(|| {
        p.scenes
            .iter()
            .position(|s| s.name.to_lowercase() == needle)
    })
}

fn find_shot_any_scene(p: &Project, ref_: &str) -> Option<(usize, usize)> {
    let needle = ref_.trim().to_lowercase();
    for (si, scene) in p.scenes.iter().enumerate() {
        if let Some(shi) = scene.shots.iter().position(|s| s.id == ref_).or_else(|| {
            scene
                .shots
                .iter()
                .position(|s| s.name.to_lowercase() == needle)
        }) {
            return Some((si, shi));
        }
    }
    None
}

fn apply_spec(spec: &mut ShotSpec, patch: &SpecPatch) {
    if let Some(v) = patch.duration_sec {
        spec.duration_sec = Some(v);
    }
    if let Some(v) = patch.fps {
        spec.fps = Some(v);
    }
    if let Some(ref v) = patch.aspect_ratio {
        spec.aspect_ratio = Some(v.clone());
    }
    if let Some(ref v) = patch.size {
        spec.size = Some(v.clone());
    }
    if let Some(ref v) = patch.angle {
        spec.angle = Some(v.clone());
    }
    if let Some(ref v) = patch.lens {
        spec.lens = Some(v.clone());
    }
    if let Some(ref v) = patch.movement {
        spec.movement = Some(v.clone());
    }
}

fn apply_defaults(d: &mut ProjectDefaults, patch: &DefaultsPatch) {
    if let Some(ref v) = patch.aspect_ratio {
        d.aspect_ratio = v.clone();
    }
    if let Some(v) = patch.fps {
        d.fps = v;
    }
    if let Some(v) = patch.duration_sec {
        d.duration_sec = v;
    }
    if let Some(ref v) = patch.target_model {
        d.target_model = v.clone();
    }
    if let Some(v) = patch.brain {
        d.brain = v;
    }
    if let Some(ref v) = patch.local_endpoint {
        d.local_endpoint = Some(v.clone());
    }
    if let Some(ref v) = patch.local_model {
        d.local_model = Some(v.clone());
    }
}

/// Mirrors `blankShot` in `src/renderer/src/stores/project.ts`.
fn blank_shot(name: &str, defaults: &ProjectDefaults) -> Shot {
    let now = now_iso();
    Shot {
        id: uid("shot"),
        name: name.to_string(),
        intent: String::new(),
        spec: ShotSpec {
            duration_sec: Some(defaults.duration_sec),
            fps: Some(defaults.fps),
            aspect_ratio: Some(defaults.aspect_ratio.clone()),
            lens: None,
            movement: None,
            size: None,
            angle: None,
        },
        prompt: String::new(),
        locked_lines: Vec::new(),
        muted_lines: Vec::new(),
        beat_sheet: None,
        target_model: Some(defaults.target_model.clone()),
        max_chars: None,
        variants: Vec::new(),
        history: Vec::new(),
        takes: Vec::new(),
        created_at: now.clone(),
        updated_at: now,
    }
}

fn parse_interior_exterior(s: &str) -> InteriorExterior {
    match s {
        "interior" => InteriorExterior::Interior,
        "both" => InteriorExterior::Both,
        _ => InteriorExterior::Exterior,
    }
}

fn parse_art_kind(s: &str) -> ArtDeptKind {
    match s {
        "wardrobe" => ArtDeptKind::Wardrobe,
        "vehicle" => ArtDeptKind::Vehicle,
        _ => ArtDeptKind::Prop,
    }
}

fn parse_vocals(s: &str) -> VocalsPreference {
    match s {
        "vocals" => VocalsPreference::Vocals,
        "either" => VocalsPreference::Either,
        _ => VocalsPreference::Instrumental,
    }
}

// ---- apply_ad_actions ----

/// Mutates `project` in place; returns human-readable receipts and UI focus.
pub fn apply_ad_actions(project: &mut Project, actions: &[AdAction]) -> ApplyResult {
    let mut receipts = Vec::new();
    let mut focus_scene_id: Option<String> = None;
    let mut focus_shot_id: Option<String> = None;

    for a in actions {
        match a {
            AdAction::UpdateProject {
                logline,
                world,
                defaults,
            } => {
                if let Some(l) = logline {
                    project.logline = l.clone();
                }
                if let Some(w) = world {
                    project.world = w.clone();
                }
                if let Some(d) = defaults {
                    apply_defaults(&mut project.defaults, d);
                }
                receipts.push("✓ Updated project bible".into());
            }

            AdAction::CreateScene { name, synopsis } => {
                if find_scene_idx(project, name).is_some() {
                    receipts.push(format!("• Scene \"{name}\" already exists — skipped"));
                    continue;
                }
                let sc = Scene {
                    id: uid("scene"),
                    name: name.clone(),
                    synopsis: synopsis.clone().unwrap_or_default(),
                    shots: Vec::new(),
                };
                focus_scene_id = Some(sc.id.clone());
                focus_shot_id = None;
                project.scenes.push(sc);
                receipts.push(format!("✓ Created scene \"{name}\""));
            }

            AdAction::UpdateScene {
                scene,
                name,
                synopsis,
            } => {
                let Some(idx) = find_scene_idx(project, scene) else {
                    receipts.push(format!("✗ Scene \"{scene}\" not found"));
                    continue;
                };
                let sc = &mut project.scenes[idx];
                if let Some(n) = name {
                    sc.name = n.clone();
                }
                if let Some(s) = synopsis {
                    sc.synopsis = s.clone();
                }
                let sc_name = sc.name.clone();
                receipts.push(format!("✓ Updated scene \"{sc_name}\""));
            }

            AdAction::CreateShot {
                scene,
                name,
                intent,
                prompt,
                spec,
                target_model,
                max_chars,
                beat_sheet,
            } => {
                let Some(idx) = find_scene_idx(project, scene) else {
                    receipts.push(format!("✗ Scene \"{scene}\" not found for new shot"));
                    continue;
                };
                let sc = &mut project.scenes[idx];
                let shot_name = name
                    .clone()
                    .unwrap_or_else(|| format!("Shot {:02}", sc.shots.len() + 1));
                let mut shot = blank_shot(&shot_name, &project.defaults);
                if let Some(i) = intent {
                    shot.intent = i.clone();
                }
                if let Some(p) = prompt {
                    shot.prompt = p.clone();
                }
                if let Some(sp) = spec {
                    apply_spec(&mut shot.spec, sp);
                }
                if let Some(tm) = target_model {
                    shot.target_model = Some(tm.clone());
                }
                if let Some(mc) = max_chars {
                    shot.max_chars = Some(*mc);
                }
                if let Some(bs) = beat_sheet {
                    shot.beat_sheet = Some(bs.clone());
                }
                let scene_id = sc.id.clone();
                let scene_name = sc.name.clone();
                let shot_id = shot.id.clone();
                let shot_display = shot.name.clone();
                let with_prompt = if prompt.is_some() { " with prompt" } else { "" };
                sc.shots.push(shot);
                focus_scene_id = Some(scene_id);
                focus_shot_id = Some(shot_id);
                receipts.push(format!(
                    "✓ Created \"{shot_display}\" in \"{scene_name}\"{with_prompt}"
                ));
            }

            AdAction::UpdateShot {
                shot: shot_ref,
                name,
                intent,
                prompt,
                spec,
                target_model,
                max_chars,
                beat_sheet,
            } => {
                let Some((si, shi)) = find_shot_any_scene(project, shot_ref) else {
                    receipts.push(format!("✗ Shot \"{shot_ref}\" not found"));
                    continue;
                };
                let scene_id = project.scenes[si].id.clone();
                let shot = &mut project.scenes[si].shots[shi];

                if let Some(new_prompt) = prompt {
                    if new_prompt != &shot.prompt && !shot.prompt.trim().is_empty() {
                        shot.history.insert(
                            0,
                            crate::types::PromptVersion {
                                id: uid("v"),
                                saved_at: now_iso(),
                                label: "before First AD change".into(),
                                prompt: shot.prompt.clone(),
                            },
                        );
                        if shot.history.len() > 50 {
                            shot.history.truncate(50);
                        }
                    }
                    shot.prompt = new_prompt.clone();
                }
                if let Some(n) = name {
                    shot.name = n.clone();
                }
                if let Some(i) = intent {
                    shot.intent = i.clone();
                }
                if let Some(sp) = spec {
                    apply_spec(&mut shot.spec, sp);
                }
                if let Some(tm) = target_model {
                    shot.target_model = Some(tm.clone());
                }
                if let Some(mc) = max_chars {
                    shot.max_chars = *mc;
                }
                if let Some(bs) = beat_sheet {
                    shot.beat_sheet = bs.clone();
                }
                shot.updated_at = now_iso();
                let shot_name = shot.name.clone();
                let shot_id = shot.id.clone();
                focus_scene_id = Some(scene_id);
                focus_shot_id = Some(shot_id);
                receipts.push(format!("✓ Updated \"{shot_name}\""));
            }

            AdAction::AddCharacter {
                name,
                age,
                gender,
                ethnicity,
                face_features,
                hair,
                clothing,
                expression,
                eye_direction,
                mood,
                environment,
                key_light_side,
                lighting_mood,
            } => {
                let display = opt_str(name);
                let key = opt_str(key_light_side);
                let light = opt_str(lighting_mood);
                project.characters.push(CharacterSheet {
                    id: uid("char"),
                    name: if display.is_empty() {
                        "Unnamed".into()
                    } else {
                        display.clone()
                    },
                    age: opt_str(age),
                    gender: opt_str(gender),
                    ethnicity: opt_str(ethnicity),
                    face_features: opt_str(face_features),
                    hair: opt_str(hair),
                    clothing: opt_str(clothing),
                    expression: opt_str(expression),
                    eye_direction: opt_str(eye_direction),
                    mood: opt_str(mood),
                    environment: opt_str(environment),
                    key_light_side: if key.is_empty() {
                        "Key light from left".into()
                    } else {
                        key
                    },
                    lighting_mood: if light.is_empty() {
                        "Natural soft light".into()
                    } else {
                        light
                    },
                    scenario: ScenarioTab::Cinematic,
                    notes: String::new(),
                    images: None,
                });
                receipts.push(format!("✓ Cast \"{display}\""));
            }

            AdAction::AddLocation {
                name,
                interior_exterior,
                description,
                time_of_day,
                weather,
                architecture,
                textures,
                practical_lights,
            } => {
                let display = opt_str(name);
                let ie = opt_str(interior_exterior);
                project.locations.push(LocationSheet {
                    id: uid("loc"),
                    name: if display.is_empty() {
                        "Unnamed".into()
                    } else {
                        display.clone()
                    },
                    interior_exterior: parse_interior_exterior(&ie),
                    description: opt_str(description),
                    time_of_day: opt_str(time_of_day),
                    weather: opt_str(weather),
                    architecture: opt_str(architecture),
                    textures: opt_str(textures),
                    practical_lights: opt_str(practical_lights),
                    notes: String::new(),
                    images: None,
                });
                receipts.push(format!("✓ Scouted \"{display}\""));
            }

            AdAction::AddArt {
                kind,
                name,
                description,
                materials,
                condition,
                era,
                distinctive,
            } => {
                let k = opt_str(kind);
                let display = opt_str(name);
                project.art_dept.push(ArtDeptSheet {
                    id: uid("art"),
                    kind: parse_art_kind(&k),
                    name: if display.is_empty() {
                        "Unnamed".into()
                    } else {
                        display.clone()
                    },
                    description: opt_str(description),
                    materials: opt_str(materials),
                    condition: opt_str(condition),
                    era: opt_str(era),
                    distinctive: opt_str(distinctive),
                    notes: String::new(),
                });
                let kind_label = if k.is_empty() { "prop" } else { k.as_str() };
                receipts.push(format!("✓ Added {kind_label} \"{display}\""));
            }

            AdAction::AddMusicCue {
                name,
                scene_ref,
                intent,
                genre,
                mood,
                tempo,
                instrumentation,
                era,
                structure,
                vocals,
                lyric_theme,
                duration_sec,
            } => {
                let display = opt_str(name);
                let vocals_s = opt_str(vocals);
                let cue = MusicCue {
                    id: uid("cue"),
                    name: if display.is_empty() {
                        "Untitled cue".into()
                    } else {
                        display.clone()
                    },
                    scene_ref: opt_str(scene_ref),
                    intent: opt_str(intent),
                    genre: opt_str(genre),
                    mood: opt_str(mood),
                    tempo: opt_str(tempo),
                    instrumentation: opt_str(instrumentation),
                    era: opt_str(era),
                    structure: opt_str(structure),
                    vocals: parse_vocals(&vocals_s),
                    lyric_theme: opt_str(lyric_theme),
                    lyrics: String::new(),
                    duration_sec: *duration_sec,
                    notes: String::new(),
                };
                project.music.get_or_insert_with(Vec::new).push(cue);
                receipts.push(format!("✓ Spotted cue \"{display}\""));
            }

            AdAction::AddVoice {
                name,
                character_name,
                age_gender,
                accent,
                timbre,
                pitch,
                pacing,
                energy,
                texture,
                emotional_range,
                sample_line,
            } => {
                let display = opt_str(name);
                let char_name = opt_str(character_name).to_lowercase();
                let character_id = if char_name.is_empty() {
                    None
                } else {
                    project
                        .characters
                        .iter()
                        .find(|c| c.name.to_lowercase() == char_name)
                        .map(|c| c.id.clone())
                };
                let voice = VoiceSheet {
                    id: uid("voice"),
                    name: if display.is_empty() {
                        "Unnamed voice".into()
                    } else {
                        display.clone()
                    },
                    character_id,
                    age_gender: opt_str(age_gender),
                    accent: opt_str(accent),
                    timbre: opt_str(timbre),
                    pitch: opt_str(pitch),
                    pacing: opt_str(pacing),
                    energy: opt_str(energy),
                    texture: opt_str(texture),
                    emotional_range: opt_str(emotional_range),
                    sample_line: opt_str(sample_line),
                    notes: String::new(),
                    grok_voice_id: None,
                    vo_text: None,
                    vo_path: None,
                    vo_language: None,
                };
                project.voices.get_or_insert_with(Vec::new).push(voice);
                receipts.push(format!("✓ Cast voice \"{display}\""));
            }

            AdAction::Select { scene, shot } => {
                if let Some(shot_ref) = shot {
                    if let Some((si, shi)) = find_shot_any_scene(project, shot_ref) {
                        focus_scene_id = Some(project.scenes[si].id.clone());
                        focus_shot_id = Some(project.scenes[si].shots[shi].id.clone());
                    }
                } else if let Some(scene_ref) = scene {
                    if let Some(idx) = find_scene_idx(project, scene_ref) {
                        let sc = &project.scenes[idx];
                        focus_scene_id = Some(sc.id.clone());
                        focus_shot_id = sc.shots.first().map(|s| s.id.clone());
                    }
                }
            }
        }
    }

    ApplyResult {
        receipts,
        focus_scene_id,
        focus_shot_id,
    }
}
