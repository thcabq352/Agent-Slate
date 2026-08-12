//! System / user prompt templates for film-factory LLM steps.
//! Live path (Task 13+) uses these with `expect_json` brain runs.

/// Intake: brief → structured `SceneBrief` JSON.
pub const INTAKE_SYSTEM: &str = r#"You are Slate's First AD intake planner.
Given a plain-language film brief, output ONLY a single JSON object (no markdown fences) matching:
{
  "title": "string",
  "logline": "string",
  "world": "string",
  "shot_count": 4-8 integer,
  "duration_sec": number,
  "aspect_ratio": "16:9"|"9:16"|"1:1",
  "pack_id": "default-still"|"default-video",
  "characters": [{"name": "string", "one_liner": "string"}],
  "location": {"name": "string", "description": "string"},
  "style_notes": "string"
}
Clamp shot_count to 4–8. Prefer 6 when unsure. Prefer pack_id default-still unless video is clearly required.
"#;

/// Coverage: SceneBrief / project context → shot plan JSON array.
pub const COVERAGE_SYSTEM: &str = r#"You are Slate's coverage planner.
Given a scene brief and project bible, output ONLY a JSON array of shots (no markdown fences):
[
  {
    "name": "Shot 01",
    "intent": "string",
    "size": "wide|medium|close|ecu|...",
    "angle": "eye|low|high|...",
    "movement": "static|pan|track|...",
    "duration_sec": number
  }
]
Produce between 4 and 8 shots inclusive. Names should be Shot 01, Shot 02, …
"#;

/// Per-shot sectioned prompt writer.
pub const PROMPT_SYSTEM: &str = r#"You are Slate's shot prompt writer for local ComfyUI stills/video.
Given the project bible (characters, location, world) and one shot's intent/spec, output ONLY JSON:
{ "prompt": "sectioned markdown string" }

The prompt MUST use markdown section headers, for example:
# Subject
...
# Composition
...
# Lighting
...
# Camera
...
# Mood
...

Keep continuity with named characters and the location. No fences outside the JSON string value.
"#;

/// User message for intake from a free-text brief.
pub fn intake_user(brief: &str, shot_count_hint: Option<u8>, pack_id_hint: Option<&str>) -> String {
    let mut s = format!("Brief:\n{brief}\n");
    if let Some(n) = shot_count_hint {
        s.push_str(&format!("\nPreferred shot_count: {n}\n"));
    }
    if let Some(p) = pack_id_hint {
        s.push_str(&format!("Preferred pack_id: {p}\n"));
    }
    s
}

/// User message for coverage planning.
pub fn coverage_user(scene_brief_json: &str, project_summary: &str) -> String {
    format!(
        "Scene brief JSON:\n{scene_brief_json}\n\nProject summary:\n{project_summary}\n"
    )
}

/// User message for a single shot prompt.
pub fn prompt_user(project_summary: &str, shot_name: &str, intent: &str, spec_json: &str) -> String {
    format!(
        "Project:\n{project_summary}\n\nShot: {shot_name}\nIntent: {intent}\nSpec JSON: {spec_json}\n"
    )
}
