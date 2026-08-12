//! Synchronous film-factory pipeline (steps 0–7).
//!
//! **Dry-run / forced stub** (`SLATE_DRY_RUN=1` or `config.dry_run`): deterministic
//! stub planner — no LLM calls.
//!
//! **Live path** (`!dry_run` and a brain backend configured via `args.brain` or
//! `config.brain_default`, and that backend is healthy): LLM steps via
//! `slate_brain::brain_run` with `expect_json`:
//! 1. Intake → [`SceneBrief`]
//! 2. Bible actions from brief
//! 3. Coverage shot list
//! 4. Per-shot sectioned prompts
//! 5. compile + generate (existing Comfy path)
//!
//! Live requires one of claude / codex / local. On mid-step brain failure the
//! factory retries once (schema nudge), then falls back to stub for that step
//! so partial projects still generate when possible.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use slate_brain::{
    brain_run, brain_status, BrainBackend as Bb, BrainRequest, BrainStatus, BrainTier,
};
use slate_comfy::{generate_to_file, ComfyClient};
use slate_domain::{
    apply_ad_actions, compile_for_comfy, create_project, open_project, save_project, uid, AdAction,
    BrainBackend, Project, SpecPatch, Take,
};

use crate::config::{apply_env, EngineConfig};
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
    /// Quality-gate verdict after generate (if run).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<slate_brain::QualityVerdict>,
    /// How many generate attempts were used (1 = first accept or single try).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempts: Option<u32>,
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

/// One planned shot from coverage LLM step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageShotPlan {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub intent: Option<String>,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub angle: Option<String>,
    #[serde(default)]
    pub movement: Option<String>,
    #[serde(default)]
    pub duration_sec: Option<f64>,
    /// Optional if the model returns a full prompt during coverage.
    #[serde(default)]
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PromptJson {
    prompt: String,
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

/// Ensure coverage plans are within V1 range 4–8: pad with stub plans if fewer
/// than 4, truncate if more than 8.
pub fn pad_coverage_plans(mut plans: Vec<CoverageShotPlan>) -> Vec<CoverageShotPlan> {
    if plans.len() > 8 {
        plans.truncate(8);
    }
    while plans.len() < 4 {
        let i = plans.len() + 1;
        let size = if i == 1 {
            "wide"
        } else if i == 4 {
            "close"
        } else {
            "medium"
        };
        plans.push(CoverageShotPlan {
            name: Some(format!("Shot {i:02}")),
            intent: Some(format!("Coverage beat {i} of 4")),
            size: Some(size.into()),
            angle: Some("eye".into()),
            movement: Some("static".into()),
            duration_sec: None,
            prompt: None,
        });
    }
    plans
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

/// Bible-only First-AD actions (project, scene, characters, location) — no shots.
pub fn bible_actions_from_brief(brief: &SceneBrief, source_brief: &str) -> Vec<AdAction> {
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
            name: scene_name,
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

    actions
}

/// Stub CreateShot actions with sectioned prompts (dry-run / coverage fallback).
pub fn stub_shot_actions(brief: &SceneBrief, source_brief: &str) -> Vec<AdAction> {
    let scene_name = brief.title.clone();
    let n = clamp_shot_count(Some(brief.shot_count)) as usize;
    let prompt = sectioned_stub_prompt(source_brief);
    let mut actions = Vec::with_capacity(n);
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

/// CreateShot actions from a live coverage plan.
/// Plans are padded/truncated to 4–8 before creating actions.
pub fn coverage_to_actions(
    plans: &[CoverageShotPlan],
    scene_name: &str,
    brief: &SceneBrief,
    source_brief: &str,
) -> Vec<AdAction> {
    let plans = pad_coverage_plans(plans.to_vec());
    let n = plans.len();
    let fallback = sectioned_stub_prompt(source_brief);
    plans
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let name = p
                .name
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| format!("Shot {:02}", i + 1));
            AdAction::CreateShot {
                scene: scene_name.to_string(),
                name: Some(name),
                intent: p
                    .intent
                    .clone()
                    .or_else(|| Some(format!("Coverage beat {} of {n}", i + 1))),
                prompt: Some(
                    p.prompt
                        .clone()
                        .filter(|s| !s.trim().is_empty())
                        .unwrap_or_else(|| fallback.clone()),
                ),
                spec: Some(SpecPatch {
                    duration_sec: Some(p.duration_sec.unwrap_or(brief.duration_sec)),
                    aspect_ratio: Some(brief.aspect_ratio.clone()),
                    size: p.size.clone().or_else(|| {
                        Some(if i == 0 {
                            "wide".into()
                        } else if i + 1 == n {
                            "close".into()
                        } else {
                            "medium".into()
                        })
                    }),
                    angle: p.angle.clone().or_else(|| Some("eye".into())),
                    movement: p.movement.clone().or_else(|| Some("static".into())),
                    ..Default::default()
                }),
                target_model: None,
                max_chars: None,
                beat_sheet: None,
            }
        })
        .collect()
}

/// Build First-AD actions from a [`SceneBrief`] (bible + coverage + stub prompts).
pub fn actions_from_brief(brief: &SceneBrief, source_brief: &str) -> Vec<AdAction> {
    let mut actions = bible_actions_from_brief(brief, source_brief);
    actions.extend(stub_shot_actions(brief, source_brief));
    actions
}

/// Parse coverage JSON: bare array or `{ "shots": [...] }`.
pub fn parse_coverage_json(v: &Value) -> Result<Vec<CoverageShotPlan>, String> {
    if v.is_array() {
        let plans: Vec<CoverageShotPlan> =
            serde_json::from_value(v.clone()).map_err(|e| e.to_string())?;
        if plans.is_empty() {
            return Err("coverage array empty".into());
        }
        return Ok(plans);
    }
    if let Some(shots) = v.get("shots") {
        let plans: Vec<CoverageShotPlan> =
            serde_json::from_value(shots.clone()).map_err(|e| e.to_string())?;
        if plans.is_empty() {
            return Err("coverage.shots empty".into());
        }
        return Ok(plans);
    }
    Err("coverage JSON must be an array or {\"shots\":[...]}".into())
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

/// Resolve preferred brain: explicit arg, else `config.brain_default` (local default).
pub fn resolve_brain_backend(args: &FilmFactoryArgs, config: &EngineConfig) -> Bb {
    match args.brain {
        Some(BrainBackend::Claude) => Bb::Claude,
        Some(BrainBackend::Codex) => Bb::Codex,
        Some(BrainBackend::Local) => Bb::Local,
        None => match config.brain_default.to_lowercase().as_str() {
            "claude" => Bb::Claude,
            "codex" => Bb::Codex,
            _ => Bb::Local,
        },
    }
}

fn backend_available(status: &BrainStatus, backend: Bb) -> bool {
    match backend {
        Bb::Claude => status.claude.available,
        Bb::Codex => status.codex.available,
        Bb::Local => status.local.available,
    }
}

fn backend_label(backend: Bb) -> &'static str {
    match backend {
        Bb::Claude => "claude",
        Bb::Codex => "codex",
        Bb::Local => "local",
    }
}

/// Choose stub vs live. Stub when dry-run, or preferred brain not healthy.
///
/// Does **not** force stub merely because `args.brain` is `None` — uses
/// `config.brain_default` (Task 12 follow-up).
async fn select_planner(
    ctx: &EngineCtx,
    args: &FilmFactoryArgs,
) -> (bool, Option<Bb>, Vec<String>) {
    let mut warnings = Vec::new();
    if ctx.config.dry_run {
        return (true, None, warnings);
    }

    let preferred = resolve_brain_backend(args, &ctx.config);
    let status = brain_status(None).await;
    if backend_available(&status, preferred) {
        return (false, Some(preferred), warnings);
    }

    warnings.push(format!(
        "brain {} not available; using stub planner (live requires claude/codex/local healthy)",
        backend_label(preferred)
    ));
    (true, None, warnings)
}

fn project_summary(project: &Project) -> String {
    let chars: Vec<String> = project
        .characters
        .iter()
        .map(|c| {
            let detail = if !c.mood.is_empty() {
                c.mood.as_str()
            } else {
                c.clothing.as_str()
            };
            format!("{}: {detail}", c.name)
        })
        .collect();
    let locs: Vec<String> = project
        .locations
        .iter()
        .map(|l| format!("{}: {}", l.name, l.description))
        .collect();
    format!(
        "title={}\nlogline={}\nworld={}\ncharacters=[{}]\nlocations=[{}]\naspect={}\nduration_sec={}",
        project.name,
        project.logline,
        project.world,
        chars.join("; "),
        locs.join("; "),
        project.defaults.aspect_ratio,
        project.defaults.duration_sec,
    )
}

/// Run brain with `expect_json`, parse typed value; on schema failure retry once with nudge.
async fn brain_expect_parsed<T, F>(
    backend: Bb,
    task: &str,
    system: &str,
    prompt: String,
    parse: F,
) -> Result<T, String>
where
    F: Fn(&Value) -> Result<T, String>,
{
    async fn once(backend: Bb, task: &str, system: &str, prompt: String) -> Result<Value, String> {
        let req = BrainRequest {
            id: uid("brain"),
            task: task.into(),
            system: system.into(),
            prompt,
            images: vec![],
            tier: BrainTier::Standard,
            expect_json: true,
            local_endpoint: None,
            local_model: None,
        };
        let result = brain_run(req, backend).await;
        if !result.ok {
            return Err(result.error.unwrap_or_else(|| {
                if result.text.is_empty() {
                    "brain run failed".into()
                } else {
                    result.text
                }
            }));
        }
        result
            .json
            .ok_or_else(|| "brain returned no JSON".to_string())
    }

    let value = once(backend, task, system, prompt.clone()).await?;
    match parse(&value) {
        Ok(t) => Ok(t),
        Err(e) => {
            let nudged = format!(
                "{prompt}\n\nIMPORTANT: Previous response failed schema validation ({e}). \
                 Respond with ONLY the requested JSON. No prose, no markdown fences."
            );
            let value2 = once(backend, task, system, nudged).await?;
            parse(&value2).map_err(|e2| format!("after retry: {e2}"))
        }
    }
}

/// Result of generate + quality gate for one shot.
#[derive(Debug, Clone)]
pub struct GenerateGateResult {
    pub path: PathBuf,
    pub quality: Option<slate_brain::QualityVerdict>,
    pub attempts: u32,
    pub gate_skipped: bool,
    pub receipts: Vec<String>,
}

/// Build a short continuity blurb from project bible + scene siblings.
pub fn continuity_context(project: &Project, scene_idx: usize, shot_idx: usize) -> String {
    let mut lines = Vec::new();
    if !project.logline.is_empty() {
        lines.push(format!("Logline: {}", project.logline));
    }
    if !project.world.is_empty() {
        lines.push(format!("World: {}", project.world));
    }
    for c in project.characters.iter().take(6) {
        let bits = [
            c.age.as_str(),
            c.gender.as_str(),
            c.face_features.as_str(),
            c.hair.as_str(),
            c.clothing.as_str(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("; ");
        if bits.is_empty() {
            lines.push(format!("Character: {}", c.name));
        } else {
            lines.push(format!("Character {}: {}", c.name, bits));
        }
    }
    if let Some(scene) = project.scenes.get(scene_idx) {
        if !scene.synopsis.is_empty() {
            lines.push(format!("Scene: {} — {}", scene.name, scene.synopsis));
        }
        for (i, sh) in scene.shots.iter().enumerate() {
            if i == shot_idx {
                continue;
            }
            let snippet = sh.intent.as_str();
            if !snippet.is_empty() {
                lines.push(format!("Other shot {}: {}", sh.name, snippet));
            }
        }
        if let Some(sh) = scene.shots.get(shot_idx) {
            if !sh.intent.is_empty() {
                lines.push(format!("This shot intent: {}", sh.intent));
            }
        }
    }
    lines.join("\n")
}

/// Compile one shot, generate via Comfy (with quality-gate retries), record take.
pub async fn generate_shot_take(
    ctx: &EngineCtx,
    project: &mut Project,
    scene_idx: usize,
    shot_idx: usize,
    pack_id: &str,
) -> Result<GenerateGateResult, String> {
    use crate::quality_gate::{
        apply_retry_hints_to_prompt, format_take_notes, judge_media, rating_for_verdict,
    };

    let aspect = project.defaults.aspect_ratio.clone();
    let max_retries = ctx.config.judge_max_retries;
    let continuity = continuity_context(project, scene_idx, shot_idx);

    let (shot_id, base_prompt) = {
        let shot = project
            .scenes
            .get(scene_idx)
            .and_then(|s| s.shots.get(shot_idx))
            .ok_or_else(|| "shot not found".to_string())?;
        (shot.id.clone(), shot.prompt.clone())
    };

    let dest_dir = project_takes_dir(&project.id, &shot_id);
    apply_env(&ctx.config);
    let client = ComfyClient::new(&ctx.config.comfy_base_url).map_err(|e| e.to_string())?;

    let mut working_prompt = base_prompt.clone();
    let mut last_hints: Vec<String> = Vec::new();
    let mut receipts = Vec::new();
    let mut last_path: Option<PathBuf> = None;
    let mut last_quality: Option<slate_brain::QualityVerdict> = None;
    let mut gate_skipped = false;
    let mut attempts: u32 = 0;

    let max_attempts = max_retries.saturating_add(1);

    for attempt in 1..=max_attempts {
        attempts = attempt;
        if !last_hints.is_empty() {
            working_prompt = apply_retry_hints_to_prompt(&base_prompt, &last_hints);
            // Persist prompt pickup on the shot for history of retries.
            if let Some(shot) = project
                .scenes
                .get_mut(scene_idx)
                .and_then(|s| s.shots.get_mut(shot_idx))
            {
                if shot.prompt != working_prompt {
                    shot.history.insert(
                        0,
                        slate_domain::PromptVersion {
                            id: uid("v"),
                            saved_at: now_iso(),
                            label: format!("quality-gate retry {attempt}"),
                            prompt: shot.prompt.clone(),
                        },
                    );
                    shot.prompt = working_prompt.clone();
                    shot.updated_at = now_iso();
                }
            }
        }

        let compiled = {
            let shot = project
                .scenes
                .get(scene_idx)
                .and_then(|s| s.shots.get(shot_idx))
                .ok_or_else(|| "shot not found".to_string())?;
            // Temporarily compile from working prompt
            let mut temp = shot.clone();
            temp.prompt = working_prompt.clone();
            compile_for_comfy(&temp, &aspect)
        };

        let mut values: HashMap<String, Value> = HashMap::new();
        values.insert("positive".into(), json!(compiled.positive));
        values.insert("negative".into(), json!(compiled.negative));
        values.insert("width".into(), json!(compiled.width));
        values.insert("height".into(), json!(compiled.height));
        // New seed each attempt (randomize also happens in inject if omitted).
        values.insert(
            "seed".into(),
            json!(rand::random::<u64>() % 1_000_000_000),
        );

        let path = generate_to_file(
            &client,
            &ctx.config.packs_dir,
            pack_id,
            &values,
            &dest_dir,
        )
        .await
        .map_err(|e| e.to_string())?;
        last_path = Some(path.clone());
        receipts.push(format!(
            "• generate attempt {attempt}/{max_attempts}: {}",
            path.display()
        ));

        let gate = judge_media(&ctx.config, &path, &working_prompt, &continuity)
            .await
            .map_err(|e| format!("quality gate: {e}"))?;
        gate_skipped = gate.skipped;
        last_quality = Some(gate.verdict.clone());
        last_hints = gate.verdict.retry_hints.clone();

        let rating = rating_for_verdict(&gate.verdict, gate.skipped);
        let notes = format_take_notes(&path, &gate);

        let shot = project
            .scenes
            .get_mut(scene_idx)
            .and_then(|s| s.shots.get_mut(shot_idx))
            .ok_or_else(|| "shot not found after generate".to_string())?;

        shot.takes.push(Take {
            id: uid("take"),
            logged_at: now_iso(),
            model: pack_id.to_string(),
            prompt: working_prompt.clone(),
            rating,
            notes,
        });
        shot.updated_at = now_iso();

        if gate.skipped {
            receipts.push(format!(
                "• quality gate skipped: {}",
                gate.skip_reason.unwrap_or_else(|| "n/a".into())
            ));
            break;
        }

        if gate.verdict.accept {
            receipts.push(format!(
                "✓ quality pass overall={:.2} (attempt {attempt})",
                gate.verdict.overall
            ));
            break;
        }

        receipts.push(format!(
            "• quality fail overall={:.2} (attempt {attempt}): {}",
            gate.verdict.overall,
            gate.verdict.summary
        ));

        if attempt >= max_attempts {
            receipts.push(format!(
                "• quality gate exhausted retries ({max_retries}); keeping last take as no-good"
            ));
            break;
        }
    }

    let path = last_path.ok_or_else(|| "no take path produced".to_string())?;
    Ok(GenerateGateResult {
        path,
        quality: last_quality,
        attempts,
        gate_skipped,
        receipts,
    })
}

/// Full pipeline: health → intake/bible/coverage/prompts → compile → generate → review.
///
/// When `ctx.config.dry_run` is true, or the resolved brain backend is unhealthy,
/// uses the deterministic stub planner (no LLM). Live path uses `args.brain` or
/// `config.brain_default` (does not require an explicit brain arg).
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

    let (use_stub, live_backend, select_warns) = select_planner(ctx, &args).await;
    warnings.extend(select_warns);

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
        if let Some(b) = live_backend {
            receipts.push(format!(
                "✓ brain: {} (live LLM path; requires claude/codex/local)",
                backend_label(b)
            ));
        } else {
            receipts.push("✓ planner: stub (dry-run or no healthy brain)".into());
        }
    }

    // Step 1 — intake
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

    let mut scene_brief = if use_stub {
        receipts.push("✓ intake: deterministic stub planner".into());
        stub_scene_brief(
            &args.brief,
            args.shot_count,
            args.pack_id.as_deref(),
        )
    } else {
        let backend = live_backend.expect("live path has backend");
        match live_intake(backend, &args).await {
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

    // Honor caller overrides after intake.
    if let Some(n) = args.shot_count {
        scene_brief.shot_count = clamp_shot_count(Some(n));
    } else {
        scene_brief.shot_count = clamp_shot_count(Some(scene_brief.shot_count));
    }
    if let Some(ref p) = args.pack_id {
        scene_brief.pack_id = p.clone();
    }

    let pack_id = scene_brief.pack_id.clone();
    let project_name = args
        .project_name
        .clone()
        .unwrap_or_else(|| scene_brief.title.clone());

    // Step 2 — bible
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

    let bible = bible_actions_from_brief(&scene_brief, &args.brief);
    let applied = apply_ad_actions(&mut project, &bible);
    receipts.extend(applied.receipts);
    receipts.push("✓ bible: project / scene / characters / location".into());

    if let Err(e) = save_project(&mut project) {
        warnings.push(format!("save after bible failed: {e}"));
    }

    let scene_id = project
        .scenes
        .first()
        .map(|s| s.id.clone())
        .unwrap_or_default();
    let scene_name = project
        .scenes
        .first()
        .map(|s| s.name.clone())
        .unwrap_or_else(|| scene_brief.title.clone());
    let scene_idx = 0usize;

    // Step 3 — coverage
    set_job(
        ctx,
        JobStatus {
            active: true,
            step: "coverage".into(),
            project_id: Some(project.id.clone()),
            message: "planning shots".into(),
        },
    );

    if !use_stub {
        let backend = live_backend.expect("live path has backend");
        match live_coverage(backend, &scene_brief, &project).await {
            Ok(plans) => {
                let actions = coverage_to_actions(&plans, &scene_name, &scene_brief, &args.brief);
                let applied = apply_ad_actions(&mut project, &actions);
                receipts.extend(applied.receipts);
                // Report actual applied shot count, not plan-list clamp alone.
                let n = project
                    .scenes
                    .get(scene_idx)
                    .map(|s| s.shots.len())
                    .unwrap_or(0);
                receipts.push(format!("✓ coverage: brain planned {n} shots"));
            }
            Err(e) => {
                warnings.push(format!("coverage brain failed ({e}); using stub shots"));
                let actions = stub_shot_actions(&scene_brief, &args.brief);
                let applied = apply_ad_actions(&mut project, &actions);
                receipts.extend(applied.receipts);
            }
        }
    } else {
        let actions = stub_shot_actions(&scene_brief, &args.brief);
        let applied = apply_ad_actions(&mut project, &actions);
        receipts.extend(applied.receipts);
        receipts.push("✓ coverage: stub shot list".into());
    }

    if let Err(e) = save_project(&mut project) {
        warnings.push(format!("save after coverage failed: {e}"));
    }

    // Step 4 — per-shot prompts (live only when brain path; stub already has prompts)
    if !use_stub {
        set_job(
            ctx,
            JobStatus {
                active: true,
                step: "prompts".into(),
                project_id: Some(project.id.clone()),
                message: "writing sectioned prompts".into(),
            },
        );
        let backend = live_backend.expect("live path has backend");
        let summary = project_summary(&project);
        let shot_count = project
            .scenes
            .get(scene_idx)
            .map(|s| s.shots.len())
            .unwrap_or(0);

        let mut prompt_ok = 0usize;
        for shot_idx in 0..shot_count {
            if ctx.cancel.load(std::sync::atomic::Ordering::SeqCst) {
                warnings.push("cancelled during prompts".into());
                break;
            }
            let (shot_id, shot_name, intent, spec_json) = {
                let shot = &project.scenes[scene_idx].shots[shot_idx];
                let spec_json = serde_json::to_string(&shot.spec).unwrap_or_else(|_| "{}".into());
                (
                    shot.id.clone(),
                    shot.name.clone(),
                    shot.intent.clone(),
                    spec_json,
                )
            };

            // Skip re-write if coverage already returned a rich prompt and we're
            // only regenerating empty/stub-looking ones when live coverage failed
            // to include prompts — always run live prompts for consistency on live path.
            match live_shot_prompt(backend, &summary, &shot_name, &intent, &spec_json).await {
                Ok(prompt) => {
                    let action = AdAction::UpdateShot {
                        shot: shot_id,
                        name: None,
                        intent: None,
                        prompt: Some(prompt),
                        spec: None,
                        target_model: None,
                        max_chars: None,
                        beat_sheet: None,
                    };
                    let applied = apply_ad_actions(&mut project, &[action]);
                    receipts.extend(applied.receipts);
                    prompt_ok += 1;
                }
                Err(e) => {
                    warnings.push(format!("prompt brain failed for {shot_name} ({e}); keeping prior"));
                }
            }
        }
        receipts.push(format!(
            "✓ prompts: brain wrote {prompt_ok}/{shot_count} sectioned prompts"
        ));

        if let Err(e) = save_project(&mut project) {
            warnings.push(format!("save after prompts failed: {e}"));
        }
    }

    set_job(
        ctx,
        JobStatus {
            active: true,
            step: "generate".into(),
            project_id: Some(project.id.clone()),
            message: format!(
                "{} shots via pack {pack_id}",
                project
                    .scenes
                    .get(0)
                    .map(|s| s.shots.len())
                    .unwrap_or(0)
            ),
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
            if let Some(scene) = project.scenes.get(scene_idx) {
                for s in scene.shots.iter().skip(shot_idx) {
                    outcomes.push(ShotOutcome {
                        id: s.id.clone(),
                        name: s.name.clone(),
                        prompt: s.prompt.clone(),
                        take_path: None,
                        error: Some("cancelled".into()),
                        quality: None,
                        attempts: None,
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
            Ok(res) => {
                receipts.extend(res.receipts);
                receipts.push(format!("✓ take for {name}: {}", res.path.display()));
                outcomes.push(ShotOutcome {
                    id,
                    name,
                    prompt,
                    take_path: Some(res.path),
                    error: None,
                    quality: res.quality,
                    attempts: Some(res.attempts),
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
                    quality: None,
                    attempts: None,
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

async fn live_intake(backend: Bb, args: &FilmFactoryArgs) -> Result<SceneBrief, String> {
    let user = prompts::intake_user(
        &args.brief,
        args.shot_count,
        args.pack_id.as_deref(),
    );
    brain_expect_parsed(backend, "intake", prompts::INTAKE_SYSTEM, user, |v| {
        serde_json::from_value::<SceneBrief>(v.clone()).map_err(|e| e.to_string())
    })
    .await
}

async fn live_coverage(
    backend: Bb,
    brief: &SceneBrief,
    project: &Project,
) -> Result<Vec<CoverageShotPlan>, String> {
    let brief_json = serde_json::to_string_pretty(brief).map_err(|e| e.to_string())?;
    let summary = project_summary(project);
    let user = prompts::coverage_user(&brief_json, &summary);
    brain_expect_parsed(
        backend,
        "coverage",
        prompts::COVERAGE_SYSTEM,
        user,
        parse_coverage_json,
    )
    .await
    .map(pad_coverage_plans)
}

async fn live_shot_prompt(
    backend: Bb,
    project_summary: &str,
    shot_name: &str,
    intent: &str,
    spec_json: &str,
) -> Result<String, String> {
    let user = prompts::prompt_user(project_summary, shot_name, intent, spec_json);
    brain_expect_parsed(backend, "prompts", prompts::PROMPT_SYSTEM, user, |v| {
        // Accept { "prompt": "..." } or a bare string.
        if let Some(s) = v.as_str() {
            if s.trim().is_empty() {
                return Err("empty prompt string".into());
            }
            return Ok(s.to_string());
        }
        let p: PromptJson = serde_json::from_value(v.clone()).map_err(|e| e.to_string())?;
        if p.prompt.trim().is_empty() {
            return Err("empty prompt field".into());
        }
        Ok(p.prompt)
    })
    .await
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
        Ok(res) => {
            save_project(&mut project).map_err(|e| e.to_string())?;
            Ok(ShotOutcome {
                id,
                name,
                prompt,
                take_path: Some(res.path),
                error: None,
                quality: res.quality,
                attempts: Some(res.attempts),
            })
        }
        Err(e) => Ok(ShotOutcome {
            id,
            name,
            prompt,
            take_path: None,
            error: Some(e),
            quality: None,
            attempts: None,
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let b: SceneBrief = serde_json::from_value(v).unwrap();
        assert_eq!(b.shot_count, 6);
        assert_eq!(b.title, "Chase");
        assert_eq!(b.characters[0].name, "Kaia");
    }

    #[test]
    fn parse_coverage_array_and_wrapped() {
        let arr = serde_json::json!([
            {"name": "Shot 01", "intent": "establish", "size": "wide"},
            {"name": "Shot 02", "intent": "detail", "size": "close"}
        ]);
        let plans = parse_coverage_json(&arr).unwrap();
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].name.as_deref(), Some("Shot 01"));

        let wrapped = serde_json::json!({
            "shots": [
                {"name": "A", "intent": "x"},
                {"name": "B", "intent": "y"},
                {"name": "C", "intent": "z"},
                {"name": "D", "intent": "w"}
            ]
        });
        let plans = parse_coverage_json(&wrapped).unwrap();
        assert_eq!(plans.len(), 4);
    }

    #[test]
    fn pad_coverage_plans_two_yields_at_least_four() {
        let arr = serde_json::json!([
            {"name": "Shot 01", "intent": "establish", "size": "wide"},
            {"name": "Shot 02", "intent": "detail", "size": "close"}
        ]);
        let plans = parse_coverage_json(&arr).unwrap();
        assert_eq!(plans.len(), 2);

        let padded = pad_coverage_plans(plans);
        assert!(padded.len() >= 4, "expected >=4, got {}", padded.len());
        assert!(padded.len() <= 8);
        assert_eq!(padded[0].name.as_deref(), Some("Shot 01"));
        assert_eq!(padded[1].name.as_deref(), Some("Shot 02"));
        // Stub pads fill the rest.
        assert!(padded[2].name.is_some());
        assert!(padded[3].name.is_some());
    }

    #[test]
    fn coverage_to_actions_two_plans_yields_four_shots() {
        let plans = vec![
            CoverageShotPlan {
                name: Some("A".into()),
                intent: Some("establish".into()),
                size: Some("wide".into()),
                angle: None,
                movement: None,
                duration_sec: None,
                prompt: None,
            },
            CoverageShotPlan {
                name: Some("B".into()),
                intent: Some("detail".into()),
                size: Some("close".into()),
                angle: None,
                movement: None,
                duration_sec: None,
                prompt: None,
            },
        ];
        let brief = stub_scene_brief("test brief", Some(4), None);
        let actions = coverage_to_actions(&plans, "Scene", &brief, "test brief");
        assert_eq!(actions.len(), 4, "post-process must pad to min 4 CreateShot actions");
        for a in &actions {
            assert!(matches!(a, AdAction::CreateShot { .. }));
        }
    }

    #[test]
    fn pad_coverage_plans_truncates_above_eight() {
        let plans: Vec<CoverageShotPlan> = (1..=12)
            .map(|i| CoverageShotPlan {
                name: Some(format!("S{i}")),
                intent: Some(format!("beat {i}")),
                size: None,
                angle: None,
                movement: None,
                duration_sec: None,
                prompt: None,
            })
            .collect();
        let padded = pad_coverage_plans(plans);
        assert_eq!(padded.len(), 8);
    }

    #[test]
    fn resolve_brain_uses_config_when_args_none() {
        let args = FilmFactoryArgs {
            brief: "x".into(),
            pack_id: None,
            brain: None,
            shot_count: None,
            project_name: None,
        };
        let mut config = EngineConfig {
            data_dir: PathBuf::from("."),
            comfy_base_url: "http://127.0.0.1:8188".into(),
            packs_dir: PathBuf::from("packs"),
            brain_default: "claude".into(),
            bind: "127.0.0.1".into(),
            dry_run: false,
            judge_model: slate_brain::DEFAULT_JUDGE_MODEL.into(),
            judge_endpoint: slate_brain::DEFAULT_OLLAMA_ENDPOINT.into(),
            judge_pass_threshold: 0.7,
            judge_max_retries: 2,
        };
        assert_eq!(resolve_brain_backend(&args, &config), Bb::Claude);
        config.brain_default = "codex".into();
        assert_eq!(resolve_brain_backend(&args, &config), Bb::Codex);
        config.brain_default = "local".into();
        assert_eq!(resolve_brain_backend(&args, &config), Bb::Local);
    }
}
