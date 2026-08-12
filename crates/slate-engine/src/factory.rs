//! Synchronous film-factory pipeline (steps 0–7).
//!
//! Dry-run / no-brain path uses a deterministic stub planner (no LLM).
//! Live brain steps are wired via `prompts` + `slate_brain` (refined in later tasks).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use slate_comfy::{generate_to_file, ComfyClient};
use slate_domain::{
    apply_ad_actions, compile_for_comfy, create_project, open_project, save_project, uid, AdAction,
    BrainBackend, Project, SpecPatch, Take, TakeRating,
};

use crate::config::apply_env;
use crate::prompts;
use crate::tools::{EngineCtx, JobStatus};

/// Args for `slate_film_factory` / [`run_film_factory`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilmFactoryArgs {
    pub brief: String,
    #[serde(default)]
    pub pack_id: Option<String>,
    #[serde(default)]
    pub brain: Option<BrainBackend>,
    #[serde(default)]
    pub shot_count: Option<u8>,
    #[serde(default)]
    pub project_name: Option<String>,
}

/// Per-shot outcome in the factory result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShotOutcome {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub take_path: Option<PathBuf>,
    pub error: Option<String>,
}

/// Result of a full film-factory run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilmFactoryResult {
    pub ok: bool,
    pub project_id: String,
    pub scene_id: String,
    pub shots: Vec<ShotOutcome>,
    pub receipts: Vec<String>,
    pub warnings: Vec<String>,
    pub elapsed_ms: u64,
}

/// Structured intake output (live path + tests).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneBrief {
    pub title: String,
    #[serde(default)]
    pub logline: String,
    #[serde(default)]
    pub world: String,
    #[serde(default = "default_shot_count")]
    pub shot_count: u8,
    #[serde(default = "default_duration")]
    pub duration_sec: f64,
    #[serde(default = "default_aspect")]
    pub aspect_ratio: String,
    #[serde(default = "default_pack")]
    pub pack_id: String,
    #[serde(default)]
    pub characters: Vec<SceneBriefCharacter>,
    #[serde(default)]
    pub location: Option<SceneBriefLocation>,
    #[serde(default)]
    pub style_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneBriefCharacter {
    pub name: String,
    #[serde(default)]
    pub one_liner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneBriefLocation {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

fn default_shot_count() -> u8 {
    4
}
fn default_duration() -> f64 {
    8.0
}
fn default_aspect() -> String {
    "16:9".into()
}
fn default_pack() -> String {
    "default-still".into()
}

/// Clamp shot count to V1 range 4–8 (default 4).
pub fn clamp_shot_count(n: Option<u8>) -> u8 {
    n.unwrap_or(4).clamp(4, 8)
}

/// Title = first 40 chars of brief (trimmed).
pub fn title_from_brief(brief: &str) -> String {
    let t: String = brief.chars().take(40).collect();
    let t = t.trim();
    if t.is_empty() {
        "Untitled Scene".into()
    } else {
        t.to_string()
    }
}

/// Deterministic stub planner (no LLM) — dry-run / tests / no-brain fallback.
pub fn stub_scene_brief(brief: &str, shot_count: Option<u8>, pack_id: Option<&str>) -> SceneBrief {
    let title = title_from_brief(brief);
    SceneBrief {
        title: title.clone(),
        logline: brief.trim().to_string(),
        world: format!("World implied by: {title}"),
        shot_count: clamp_shot_count(shot_count),
        duration_sec: 8.0,
        aspect_ratio: "16:9".into(),
        pack_id: pack_id.unwrap_or("default-still").to_string(),
        characters: vec![SceneBriefCharacter {
            name: "Protagonist".into(),
            one_liner: "Lead figure in the brief".into(),
        }],
        location: Some(SceneBriefLocation {
            name: "Primary Location".into(),
            description: brief.trim().to_string(),
        }),
        style_notes: "Cinematic".into(),
    }
}

fn sectioned_stub_prompt(brief: &str) -> String {
    format!("# Subject\n{brief}\n# Mood\nCinematic\n")
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    // RFC3339-ish millisecond stamp without pulling chrono into the engine crate.
    format!("{ms}")
}

/// Build First-AD actions from a [`SceneBrief`] (bible + coverage + stub prompts).
pub fn actions_from_brief(brief: &SceneBrief, source_brief: &str) -> Vec<AdAction> {
    let scene_name = brief.title.clone();
    let mut actions = vec![
        AdAction::UpdateProject {
            logline: Some(if brief.logline.is_empty() {
                source_brief.to_string()
            } else {
                brief.logline.clone()
            }),
            world: Some(brief.world.clone()),
            defaults: Some(slate_domain::DefaultsPatch {
                aspect_ratio: Some(brief.aspect_ratio.clone()),
                duration_sec: Some(brief.duration_sec),
                ..Default::default()
            }),
        },
        AdAction::CreateScene {
            name: scene_name.clone(),
            synopsis: Some(brief.logline.clone()),
        },
    ];

    if brief.characters.is_empty() {
        actions.push(AdAction::AddCharacter {
            name: Some("Protagonist".into()),
            age: None,
            gender: None,
            ethnicity: None,
            face_features: None,
            hair: None,
            clothing: None,
            expression: None,
            eye_direction: None,
            mood: None,
            environment: None,
            key_light_side: None,
            lighting_mood: None,
        });
    } else {
        for c in &brief.characters {
            actions.push(AdAction::AddCharacter {
                name: Some(c.name.clone()),
                age: None,
                gender: None,
                ethnicity: None,
                face_features: None,
                hair: None,
                clothing: None,
                expression: None,
                eye_direction: None,
                mood: Some(c.one_liner.clone()),
                environment: None,
                key_light_side: None,
                lighting_mood: None,
            });
        }
    }

    if let Some(ref loc) = brief.location {
        actions.push(AdAction::AddLocation {
            name: Some(loc.name.clone()),
            interior_exterior: None,
            description: Some(loc.description.clone()),
            time_of_day: None,
            weather: None,
            architecture: None,
            textures: None,
            practical_lights: None,
        });
    } else {
        actions.push(AdAction::AddLocation {
            name: Some("Primary Location".into()),
            interior_exterior: None,
            description: Some(source_brief.to_string()),
            time_of_day: None,
            weather: None,
            architecture: None,
            textures: None,
            practical_lights: None,
        });
    }

    let n = clamp_shot_count(Some(brief.shot_count)) as usize;
    let prompt = sectioned_stub_prompt(source_brief);
    for i in 1..=n {
        actions.push(AdAction::CreateShot {
            scene: scene_name.clone(),
            name: Some(format!("Shot {i:02}")),
            intent: Some(format!("Coverage beat {i} of {n}")),
            prompt: Some(prompt.clone()),
            spec: Some(SpecPatch {
                duration_sec: Some(brief.duration_sec),
                aspect_ratio: Some(brief.aspect_ratio.clone()),
                size: Some(if i == 1 {
                    "wide".into()
                } else if i == n {
                    "close".into()
                } else {
                    "medium".into()
                }),
                angle: Some("eye".into()),
                movement: Some("static".into()),
                ..Default::default()
            }),
            target_model: None,
            max_chars: None,
            beat_sheet: None,
        });
    }

    actions
}

fn set_job(ctx: &EngineCtx, status: JobStatus) {
    if let Ok(mut g) = ctx.job.lock() {
        *g = status;
    }
}

fn project_takes_dir(project_id: &str, shot_id: &str) -> PathBuf {
    slate_domain::projects_root()
        .join(project_id)
        .join("takes")
        .join(shot_id)
}

/// Compile one shot and write a take via Comfy (or dry-run marker).
pub async fn generate_shot_take(
    ctx: &EngineCtx,
    project: &mut Project,
    scene_idx: usize,
    shot_idx: usize,
    pack_id: &str,
) -> Result<PathBuf, String> {
    let aspect = project.defaults.aspect_ratio.clone();
    let (shot_id, compiled, prompt_text) = {
        let shot = project
            .scenes
            .get(scene_idx)
            .and_then(|s| s.shots.get(shot_idx))
            .ok_or_else(|| "shot not found".to_string())?;
        let compiled = compile_for_comfy(shot, &aspect);
        (shot.id.clone(), compiled, shot.prompt.clone())
    };

    let dest_dir = project_takes_dir(&project.id, &shot_id);
    let mut values: HashMap<String, Value> = HashMap::new();
    values.insert("positive".into(), json!(compiled.positive));
    values.insert("negative".into(), json!(compiled.negative));
    values.insert("width".into(), json!(compiled.width));
    values.insert("height".into(), json!(compiled.height));

    // Ensure dry-run env matches config for comfy helper.
    apply_env(&ctx.config);

    let client = ComfyClient::new(&ctx.config.comfy_base_url).map_err(|e| e.to_string())?;
    let path = generate_to_file(
        &client,
        &ctx.config.packs_dir,
        pack_id,
        &values,
        &dest_dir,
    )
    .await
    .map_err(|e| e.to_string())?;

    let shot = project
        .scenes
        .get_mut(scene_idx)
        .and_then(|s| s.shots.get_mut(shot_idx))
        .ok_or_else(|| "shot not found after generate".to_string())?;

    shot.takes.push(Take {
        id: uid("take"),
        logged_at: now_iso(),
        model: pack_id.to_string(),
        prompt: prompt_text,
        rating: TakeRating::Good,
        notes: path.display().to_string(),
    });
    shot.updated_at = now_iso();

    Ok(path)
}

/// Full pipeline: health → intake/bible/coverage/prompts → compile → generate → review.
///
/// When `ctx.config.dry_run` is true **or** no brain is selected/available, uses the
/// deterministic stub planner (no LLM). Blocks until all shots finish or cancel.
pub async fn run_film_factory(ctx: &EngineCtx, args: FilmFactoryArgs) -> FilmFactoryResult {
    let start = Instant::now();
    let mut receipts = Vec::new();
    let mut warnings = Vec::new();

    ctx.cancel.store(false, std::sync::atomic::Ordering::SeqCst);
    set_job(
        ctx,
        JobStatus {
            active: true,
            step: "health".into(),
            project_id: None,
            message: "starting film factory".into(),
        },
    );

    apply_env(&ctx.config);

    let use_stub = ctx.config.dry_run || args.brain.is_none();

    // Step 0 — health (relaxed in dry-run)
    if ctx.config.dry_run {
        receipts.push("✓ health: dry-run (skipped Comfy/brain probes)".into());
    } else {
        match ComfyClient::new(&ctx.config.comfy_base_url) {
            Ok(client) => match client.health().await {
                Ok(()) => receipts.push(format!("✓ Comfy healthy at {}", ctx.config.comfy_base_url)),
                Err(e) => {
                    warnings.push(format!("Comfy health failed: {e}"));
                    if !use_stub {
                        set_job(
                            ctx,
                            JobStatus {
                                active: false,
                                step: "health".into(),
                                project_id: None,
                                message: format!("comfy down: {e}"),
                            },
                        );
                        return FilmFactoryResult {
                            ok: false,
                            project_id: String::new(),
                            scene_id: String::new(),
                            shots: vec![],
                            receipts,
                            warnings,
                            elapsed_ms: start.elapsed().as_millis() as u64,
                        };
                    }
                }
            },
            Err(e) => warnings.push(format!("Comfy client error: {e}")),
        }
    }

    // Step 1 — intake (stub or live)
    set_job(
        ctx,
        JobStatus {
            active: true,
            step: "intake".into(),
            project_id: None,
            message: if use_stub {
                "stub planner".into()
            } else {
                "brain intake".into()
            },
        },
    );

    let scene_brief = if use_stub {
        receipts.push("✓ intake: deterministic stub planner".into());
        stub_scene_brief(
            &args.brief,
            args.shot_count,
            args.pack_id.as_deref(),
        )
    } else {
        match live_intake(ctx, &args).await {
            Ok(b) => {
                receipts.push("✓ intake: brain SceneBrief".into());
                b
            }
            Err(e) => {
                warnings.push(format!("intake brain failed ({e}); falling back to stub"));
                stub_scene_brief(
                    &args.brief,
                    args.shot_count,
                    args.pack_id.as_deref(),
                )
            }
        }
    };

    let pack_id = args
        .pack_id
        .clone()
        .unwrap_or_else(|| scene_brief.pack_id.clone());
    let project_name = args
        .project_name
        .clone()
        .unwrap_or_else(|| scene_brief.title.clone());

    // Step 2–4 — bible + coverage + prompts via AD actions
    set_job(
        ctx,
        JobStatus {
            active: true,
            step: "bible".into(),
            project_id: None,
            message: format!("creating project {project_name}"),
        },
    );

    let mut project = match create_project(&project_name) {
        Ok(p) => p,
        Err(e) => {
            warnings.push(format!("create_project failed: {e}"));
            set_job(
                ctx,
                JobStatus {
                    active: false,
                    step: "bible".into(),
                    project_id: None,
                    message: format!("create failed: {e}"),
                },
            );
            return FilmFactoryResult {
                ok: false,
                project_id: String::new(),
                scene_id: String::new(),
                shots: vec![],
                receipts,
                warnings,
                elapsed_ms: start.elapsed().as_millis() as u64,
            };
        }
    };

    let actions = actions_from_brief(&scene_brief, &args.brief);
    let applied = apply_ad_actions(&mut project, &actions);
    receipts.extend(applied.receipts);

    if let Err(e) = save_project(&mut project) {
        warnings.push(format!("save after bible failed: {e}"));
    }

    let scene_id = project
        .scenes
        .first()
        .map(|s| s.id.clone())
        .unwrap_or_default();
    let scene_idx = 0usize;

    set_job(
        ctx,
        JobStatus {
            active: true,
            step: "generate".into(),
            project_id: Some(project.id.clone()),
            message: format!("{} shots via pack {pack_id}", project.scenes.get(0).map(|s| s.shots.len()).unwrap_or(0)),
        },
    );

    // Steps 5–6 — compile + generate per shot
    let mut outcomes: Vec<ShotOutcome> = Vec::new();
    let shot_count = project
        .scenes
        .get(scene_idx)
        .map(|s| s.shots.len())
        .unwrap_or(0);

    for shot_idx in 0..shot_count {
        if ctx.cancel.load(std::sync::atomic::Ordering::SeqCst) {
            warnings.push("cancelled between shots".into());
            receipts.push("• cancel acknowledged — stopping further generates".into());
            // Record remaining shots without takes
            if let Some(scene) = project.scenes.get(scene_idx) {
                for s in scene.shots.iter().skip(shot_idx) {
                    outcomes.push(ShotOutcome {
                        id: s.id.clone(),
                        name: s.name.clone(),
                        prompt: s.prompt.clone(),
                        take_path: None,
                        error: Some("cancelled".into()),
                    });
                }
            }
            break;
        }

        let (id, name, prompt) = {
            let shot = &project.scenes[scene_idx].shots[shot_idx];
            (shot.id.clone(), shot.name.clone(), shot.prompt.clone())
        };

        set_job(
            ctx,
            JobStatus {
                active: true,
                step: "generate".into(),
                project_id: Some(project.id.clone()),
                message: format!("generating {name}"),
            },
        );

        match generate_shot_take(ctx, &mut project, scene_idx, shot_idx, &pack_id).await {
            Ok(path) => {
                receipts.push(format!("✓ take for {name}: {}", path.display()));
                outcomes.push(ShotOutcome {
                    id,
                    name,
                    prompt,
                    take_path: Some(path),
                    error: None,
                });
            }
            Err(e) => {
                warnings.push(format!("generate failed for {name}: {e}"));
                outcomes.push(ShotOutcome {
                    id,
                    name,
                    prompt,
                    take_path: None,
                    error: Some(e),
                });
            }
        }

        if let Err(e) = save_project(&mut project) {
            warnings.push(format!("save after shot failed: {e}"));
        }
    }

    // Step 7 — review
    let takes_ok = outcomes.iter().filter(|s| s.take_path.is_some()).count();
    let ok = takes_ok > 0 && outcomes.iter().all(|s| s.error.as_deref() != Some("fatal"));
    // Prefer ok true when ≥1 take (design); still false if zero takes.
    let ok = ok && takes_ok >= 1;

    receipts.push(format!(
        "✓ review: {takes_ok}/{} takes, project {}",
        outcomes.len(),
        project.id
    ));

    set_job(
        ctx,
        JobStatus {
            active: false,
            step: "review".into(),
            project_id: Some(project.id.clone()),
            message: if ok {
                "complete".into()
            } else {
                "finished with failures".into()
            },
        },
    );

    FilmFactoryResult {
        ok,
        project_id: project.id,
        scene_id,
        shots: outcomes,
        receipts,
        warnings,
        elapsed_ms: start.elapsed().as_millis() as u64,
    }
}

async fn live_intake(ctx: &EngineCtx, args: &FilmFactoryArgs) -> Result<SceneBrief, String> {
    use slate_brain::{brain_run, BrainBackend as Bb, BrainRequest, BrainTier};

    let backend = match args.brain {
        Some(BrainBackend::Claude) => Bb::Claude,
        Some(BrainBackend::Codex) => Bb::Codex,
        Some(BrainBackend::Local) | None => {
            // Prefer configured default
            match ctx.config.brain_default.to_lowercase().as_str() {
                "claude" => Bb::Claude,
                "codex" => Bb::Codex,
                _ => Bb::Local,
            }
        }
    };

    let req = BrainRequest {
        id: uid("brain"),
        task: "intake".into(),
        system: prompts::INTAKE_SYSTEM.into(),
        prompt: prompts::intake_user(
            &args.brief,
            args.shot_count,
            args.pack_id.as_deref(),
        ),
        images: vec![],
        tier: BrainTier::Standard,
        expect_json: true,
        local_endpoint: None,
        local_model: None,
    };

    let result = brain_run(req, backend).await;
    if !result.ok {
        return Err(result.error.unwrap_or_else(|| result.text.clone()));
    }
    let value = result
        .json
        .ok_or_else(|| "brain returned no JSON".to_string())?;
    serde_json::from_value(value).map_err(|e| e.to_string())
}

/// Re-generate a single shot (tool: `slate_generate_shot`).
pub async fn generate_one_shot(
    ctx: &EngineCtx,
    project_id: &str,
    shot_id: &str,
    pack_id: Option<&str>,
) -> Result<ShotOutcome, String> {
    apply_env(&ctx.config);
    let mut project = open_project(project_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;

    let mut found: Option<(usize, usize)> = None;
    for (si, scene) in project.scenes.iter().enumerate() {
        for (shi, shot) in scene.shots.iter().enumerate() {
            if shot.id == shot_id || shot.name == shot_id {
                found = Some((si, shi));
                break;
            }
        }
        if found.is_some() {
            break;
        }
    }
    let (scene_idx, shot_idx) = found.ok_or_else(|| "Shot not found".to_string())?;
    let pack = pack_id.unwrap_or("default-still");

    let (id, name, prompt) = {
        let s = &project.scenes[scene_idx].shots[shot_idx];
        (s.id.clone(), s.name.clone(), s.prompt.clone())
    };

    match generate_shot_take(ctx, &mut project, scene_idx, shot_idx, pack).await {
        Ok(path) => {
            save_project(&mut project).map_err(|e| e.to_string())?;
            Ok(ShotOutcome {
                id,
                name,
                prompt,
                take_path: Some(path),
                error: None,
            })
        }
        Err(e) => Ok(ShotOutcome {
            id,
            name,
            prompt,
            take_path: None,
            error: Some(e),
        }),
    }
}

/// List take media for a project (optional shot filter).
pub fn list_takes(project_id: &str, shot_id: Option<&str>) -> Result<Value, String> {
    let project = open_project(project_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;

    let mut rows = Vec::new();
    for scene in &project.scenes {
        for shot in &scene.shots {
            if let Some(filter) = shot_id {
                if shot.id != filter && shot.name != filter {
                    continue;
                }
            }
            for take in &shot.takes {
                rows.push(json!({
                    "shotId": shot.id,
                    "shotName": shot.name,
                    "takeId": take.id,
                    "loggedAt": take.logged_at,
                    "model": take.model,
                    "path": take.notes,
                    "rating": take.rating,
                }));
            }
            // Also surface files on disk under takes/{shotId}/
            let dir = project_takes_dir(&project.id, &shot.id);
            if dir.is_dir() {
                if let Ok(rd) = std::fs::read_dir(&dir) {
                    for ent in rd.flatten() {
                        let p = ent.path();
                        if p.is_file() {
                            let already = rows.iter().any(|r| {
                                r.get("path")
                                    .and_then(|v| v.as_str())
                                    .map(|s| Path::new(s) == p.as_path())
                                    .unwrap_or(false)
                            });
                            if !already {
                                rows.push(json!({
                                    "shotId": shot.id,
                                    "shotName": shot.name,
                                    "takeId": null,
                                    "loggedAt": null,
                                    "model": null,
                                    "path": p.display().to_string(),
                                    "rating": null,
                                }));
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(json!({ "projectId": project_id, "takes": rows }))
}
