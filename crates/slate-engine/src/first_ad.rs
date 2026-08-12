//! Engine First AD — conversational scene operator (Phase 3).
//!
//! Plans and mutates projects via the same AdAction contract as the Electron First AD,
//! with optional scene continuity book for downstream generate/judge.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use slate_brain::{
    brain_run, brain_status, extract_json, BrainBackend as Bb, BrainRequest, BrainTier,
};
use slate_domain::{apply_ad_actions, open_project, save_project, AdAction, ApplyResult, Project};

use crate::config::EngineConfig;
use crate::continuity::SceneContinuityContext;
use crate::factory::resolve_brain_backend;
use crate::tools::EngineCtx;

const AD_SYSTEM: &str = r#"You are the First AD inside Slate (engine). The director talks about a scene;
you reply briefly and emit actions only when intent is clear enough to act.

Respond with ONLY JSON (no markdown fences):
{
  "reply": "what you say to the director",
  "actions": [ ... ],
  "continuity_locks": ["optional standing continuity facts to lock"],
  "scene_plan_summary": "one-line plan of the set right now"
}

Available actions (type field snake_case):
- {"type":"update_project","logline"?,"world"?,"defaults"?:{...}}
- {"type":"create_scene","name","synopsis"?}
- {"type":"update_scene","scene","name"?,"synopsis"?}
- {"type":"create_shot","scene","name"?,"intent"?,"prompt"?,"spec"?:{...},"targetModel"?}
- {"type":"update_shot","shot","name"?,"intent"?,"prompt"?,"spec"?,"targetModel"?}
- {"type":"add_character","name",...}
- {"type":"add_location","name",...}
- {"type":"select","scene"?,"shot"?}

Prompt format for any "prompt" field — sectioned:
# Subject
...
# Composition
...
# Lighting
...
# Camera
...
# Style
...
# Mood
...

If intent is vague, ask 1–2 questions and return "actions": [].
When you act, do not paste full prompts into "reply".
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstAdArgs {
    pub project_id: String,
    pub message: String,
    /// Optional chat history (role + text).
    #[serde(default)]
    pub history: Vec<FirstAdChatMsg>,
    /// Optional brain override.
    #[serde(default)]
    pub brain: Option<slate_domain::BrainBackend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstAdChatMsg {
    pub role: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstAdResult {
    pub ok: bool,
    pub reply: String,
    pub receipts: Vec<String>,
    pub actions_applied: usize,
    pub scene_plan_summary: String,
    pub continuity: SceneContinuityContext,
    pub focus_scene_id: Option<String>,
    pub focus_shot_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdModelReply {
    #[serde(default)]
    reply: String,
    #[serde(default)]
    actions: Vec<Value>,
    #[serde(default)]
    continuity_locks: Vec<String>,
    #[serde(default)]
    scene_plan_summary: String,
}

fn project_inventory(p: &Project) -> String {
    let mut lines = vec![format!("PROJECT: {} [{}]", p.name, p.id)];
    if !p.logline.is_empty() {
        lines.push(format!("Logline: {}", p.logline));
    }
    if !p.world.is_empty() {
        lines.push(format!("World: {}", p.world));
    }
    lines.push("SCENES/SHOTS:".into());
    if p.scenes.is_empty() {
        lines.push("- (none)".into());
    }
    for sc in &p.scenes {
        lines.push(format!(
            "- Scene \"{}\" [{}] {}",
            sc.name, sc.id, sc.synopsis
        ));
        for sh in &sc.shots {
            lines.push(format!(
                "    · Shot \"{}\" [{}] intent={}",
                sh.name, sh.id, sh.intent
            ));
        }
    }
    lines.push("CHARACTERS:".into());
    for c in &p.characters {
        lines.push(format!("- {} clothing={}", c.name, c.clothing));
    }
    lines.push("LOCATIONS:".into());
    for l in &p.locations {
        lines.push(format!(
            "- {} weather={} time={}",
            l.name, l.weather, l.time_of_day
        ));
    }
    lines.join("\n")
}

fn parse_actions(raw: &[Value]) -> Vec<AdAction> {
    raw.iter()
        .filter_map(|v| serde_json::from_value::<AdAction>(v.clone()).ok())
        .collect()
}

fn pick_scene_idx(project: &Project, focus_scene: Option<&str>) -> usize {
    if let Some(ref_id) = focus_scene {
        if let Some((i, _)) = project
            .scenes
            .iter()
            .enumerate()
            .find(|(_, s)| s.id == ref_id || s.name.eq_ignore_ascii_case(ref_id))
        {
            return i;
        }
    }
    0
}

/// Run one First AD conversational turn against a project.
pub async fn run_first_ad(ctx: &EngineCtx, args: FirstAdArgs) -> FirstAdResult {
    let mut project = match open_project(&args.project_id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return FirstAdResult {
                ok: false,
                reply: String::new(),
                receipts: vec![],
                actions_applied: 0,
                scene_plan_summary: String::new(),
                continuity: SceneContinuityContext::default(),
                focus_scene_id: None,
                focus_shot_id: None,
                error: Some("Project not found".into()),
            };
        }
        Err(e) => {
            return FirstAdResult {
                ok: false,
                reply: String::new(),
                receipts: vec![],
                actions_applied: 0,
                scene_plan_summary: String::new(),
                continuity: SceneContinuityContext::default(),
                focus_scene_id: None,
                focus_shot_id: None,
                error: Some(e.to_string()),
            };
        }
    };

    let factory_args = crate::factory::FilmFactoryArgs {
        brief: args.message.clone(),
        pack_id: None,
        brain: args.brain,
        shot_count: None,
        project_name: None,
    };
    let backend = resolve_brain_backend(&factory_args, &ctx.config);

    // Health check for chosen backend
    let status = brain_status(None).await;
    let healthy = match backend {
        Bb::Claude => status.claude.available,
        Bb::Codex => status.codex.available,
        Bb::Local => status.local.available,
    };
    if !healthy && !ctx.config.dry_run {
        return FirstAdResult {
            ok: false,
            reply: "Brain offline — start Ollama/Claude/Codex or set a healthy brain.".into(),
            receipts: vec![],
            actions_applied: 0,
            scene_plan_summary: String::new(),
            continuity: SceneContinuityContext::from_project_scene(&project, 0),
            focus_scene_id: None,
            focus_shot_id: None,
            error: Some("brain not available".into()),
        };
    }

    // Dry-run / no brain: heuristic stub AD
    if ctx.config.dry_run || !healthy {
        return stub_first_ad(&mut project, &args);
    }

    let mut continuity = SceneContinuityContext::from_project_scene(&project, 0);
    let inventory = project_inventory(&project);
    let memory = crate::notes::notes_prompt_block(&project.id, 12);
    let hist = args
        .history
        .iter()
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|m| format!("{}: {}", m.role.to_uppercase(), m.text))
        .collect::<Vec<_>>()
        .join("\n\n");

    let user = format!(
        "{inventory}\n\nCONTINUITY BOOK:\n{}\n\n{}\n{}\n\nDIRECTOR: {}\n",
        continuity.as_prompt_block(),
        memory,
        if hist.is_empty() {
            String::new()
        } else {
            format!("CONVERSATION:\n{hist}\n")
        },
        args.message
    );

    let req = BrainRequest {
        id: format!("first-ad-{}", chrono_ms()),
        task: "first-ad".into(),
        system: AD_SYSTEM.into(),
        prompt: user,
        images: vec![],
        tier: BrainTier::Top,
        expect_json: true,
        local_endpoint: None,
        local_model: None,
    };

    let result = brain_run(req, backend).await;
    if !result.ok {
        return FirstAdResult {
            ok: false,
            reply: result
                .error
                .clone()
                .unwrap_or_else(|| "First AD brain failed".into()),
            receipts: vec![],
            actions_applied: 0,
            scene_plan_summary: String::new(),
            continuity,
            focus_scene_id: None,
            focus_shot_id: None,
            error: result.error,
        };
    }

    let value = match result.json {
        Some(j) => j,
        None => match extract_json(&result.text) {
            Ok(j) => j,
            Err(e) => {
                return FirstAdResult {
                    ok: false,
                    reply: result.text,
                    receipts: vec![],
                    actions_applied: 0,
                    scene_plan_summary: String::new(),
                    continuity,
                    focus_scene_id: None,
                    focus_shot_id: None,
                    error: Some(format!("JSON parse: {e}")),
                };
            }
        },
    };

    let parsed: AdModelReply = match serde_json::from_value(value) {
        Ok(p) => p,
        Err(e) => {
            return FirstAdResult {
                ok: false,
                reply: result.text,
                receipts: vec![],
                actions_applied: 0,
                scene_plan_summary: String::new(),
                continuity,
                focus_scene_id: None,
                focus_shot_id: None,
                error: Some(format!("schema: {e}")),
            };
        }
    };

    let actions = parse_actions(&parsed.actions);
    let ApplyResult {
        receipts,
        focus_scene_id,
        focus_shot_id,
    } = apply_ad_actions(&mut project, &actions);

    for lock in parsed.continuity_locks {
        let t = lock.trim().to_string();
        if !t.is_empty() && !continuity.locks.iter().any(|x| x == &t) {
            continuity.locks.push(t);
        }
    }

    // Rebuild continuity ids after mutations
    let scene_idx = pick_scene_idx(&project, focus_scene_id.as_deref());
    let mut cont = SceneContinuityContext::from_project_scene(&project, scene_idx);
    cont.locks.extend(continuity.locks);
    cont.standing_orders.extend(continuity.standing_orders);
    cont.beats = continuity.beats;

    if let Err(e) = save_project(&mut project) {
        return FirstAdResult {
            ok: false,
            reply: parsed.reply,
            receipts,
            actions_applied: actions.len(),
            scene_plan_summary: parsed.scene_plan_summary,
            continuity: cont,
            focus_scene_id,
            focus_shot_id,
            error: Some(format!("save failed: {e}")),
        };
    }

    let mut receipts = receipts;
    let plan_summary = if parsed.scene_plan_summary.is_empty() {
        parsed.reply.chars().take(80).collect::<String>()
    } else {
        parsed.scene_plan_summary.clone()
    };
    if crate::notes::note_scene_plan(
        &project.id,
        Some(cont.scene_id.as_str()),
        &plan_summary,
        &format!("{}\n\n{}", parsed.reply, cont.as_prompt_block()),
    )
    .is_ok()
    {
        receipts.push("• note: scene_plan recorded".into());
    }
    for lock in &cont.locks {
        let _ = crate::notes::note_continuity(
            &project.id,
            Some(&cont.scene_id),
            "continuity lock",
            lock,
        );
    }

    FirstAdResult {
        ok: true,
        reply: if parsed.reply.is_empty() {
            "Done.".into()
        } else {
            parsed.reply
        },
        receipts,
        actions_applied: actions.len(),
        scene_plan_summary: parsed.scene_plan_summary,
        continuity: cont,
        focus_scene_id,
        focus_shot_id,
        error: None,
    }
}

fn stub_first_ad(project: &mut Project, args: &FirstAdArgs) -> FirstAdResult {
    use slate_domain::AdAction;
    let scene_name = "Scene 01".to_string();
    let mut actions = vec![];
    if project.scenes.is_empty() {
        actions.push(AdAction::CreateScene {
            name: scene_name.clone(),
            synopsis: Some(args.message.chars().take(120).collect()),
        });
        actions.push(AdAction::CreateShot {
            scene: scene_name.clone(),
            name: Some("Shot 01".into()),
            intent: Some(args.message.chars().take(80).collect()),
            prompt: Some(format!(
                "# Subject\n{}\n\n# Mood\nCinematic\n",
                args.message.chars().take(200).collect::<String>()
            )),
            spec: None,
            target_model: None,
            max_chars: None,
            beat_sheet: None,
        });
    }
    let applied = apply_ad_actions(project, &actions);
    let _ = save_project(project);
    let continuity = SceneContinuityContext::from_project_scene(project, 0);
    FirstAdResult {
        ok: true,
        reply: if actions.is_empty() {
            "Stub AD: scene already exists — say what to change.".into()
        } else {
            "Stub AD: created a scene and first shot from your note (dry-run / offline brain)."
                .into()
        },
        receipts: applied.receipts,
        actions_applied: actions.len(),
        scene_plan_summary: "stub single-shot setup".into(),
        continuity,
        focus_scene_id: applied.focus_scene_id,
        focus_shot_id: applied.focus_shot_id,
        error: None,
    }
}

fn chrono_ms() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Used by tools to avoid unused import warnings on EngineConfig in this module.
#[allow(dead_code)]
fn _cfg_touch(_: &EngineConfig) {}
