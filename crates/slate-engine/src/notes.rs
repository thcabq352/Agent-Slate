//! Atomic Notes — project-local continuity / decision / quality memory (Phase 4).
//!
//! Notes live under `{projectDir}/.notes/notes.jsonl` (append-only JSON lines).
//! Optional export/sync to Hermes-style memory is out of scope for this slice;
//! the file format is plain enough to copy into `~/.hermes/memories` later.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use slate_domain::{projects_root, uid};

/// Kind of memory note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteKind {
    Continuity,
    ShotDecision,
    QualityFeedback,
    ScenePlan,
    General,
}

impl NoteKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NoteKind::Continuity => "continuity",
            NoteKind::ShotDecision => "shot_decision",
            NoteKind::QualityFeedback => "quality_feedback",
            NoteKind::ScenePlan => "scene_plan",
            NoteKind::General => "general",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "continuity" => NoteKind::Continuity,
            "shot_decision" | "shot-decision" | "shot" => NoteKind::ShotDecision,
            "quality_feedback" | "quality" | "judge" => NoteKind::QualityFeedback,
            "scene_plan" | "plan" | "first_ad" => NoteKind::ScenePlan,
            _ => NoteKind::General,
        }
    }
}

/// One atomic note record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub project_id: String,
    pub kind: NoteKind,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shot_id: Option<String>,
}

/// Args for writing a note.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteWriteArgs {
    pub project_id: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub scene_id: Option<String>,
    #[serde(default)]
    pub shot_id: Option<String>,
}

/// Args for searching notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSearchArgs {
    pub project_id: String,
    /// Free-text match against title/body/tags (case-insensitive).
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub shot_id: Option<String>,
    #[serde(default)]
    pub scene_id: Option<String>,
    /// Max hits (default 20, max 100).
    #[serde(default)]
    pub limit: Option<usize>,
}

fn now_iso() -> String {
    // Prefer chrono if domain has it; use system time RFC-ish
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    // Stable sortable timestamp without extra deps in this module path
    format!("{ms}")
}

fn project_notes_dir(project_id: &str) -> PathBuf {
    projects_root().join(project_id).join(".notes")
}

fn notes_file(project_id: &str) -> PathBuf {
    project_notes_dir(project_id).join("notes.jsonl")
}

/// Ensure `.notes` exists.
pub fn ensure_notes_dir(project_id: &str) -> Result<PathBuf, String> {
    let dir = project_notes_dir(project_id);
    fs::create_dir_all(&dir).map_err(|e| format!("create notes dir: {e}"))?;
    Ok(dir)
}

/// Append one note; returns the stored record.
pub fn write_note(args: NoteWriteArgs) -> Result<Note, String> {
    ensure_notes_dir(&args.project_id)?;
    let note = Note {
        id: uid("note"),
        project_id: args.project_id.clone(),
        kind: NoteKind::parse(&args.kind),
        title: args.title.trim().to_string(),
        body: args.body.trim().to_string(),
        tags: args.tags,
        created_at: now_iso(),
        scene_id: args.scene_id,
        shot_id: args.shot_id,
    };
    if note.title.is_empty() && note.body.is_empty() {
        return Err("note title or body required".into());
    }
    let path = notes_file(&args.project_id);
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("open notes file: {e}"))?;
    let line = serde_json::to_string(&note).map_err(|e| e.to_string())?;
    writeln!(f, "{line}").map_err(|e| format!("write note: {e}"))?;
    Ok(note)
}

/// Load all notes for a project (oldest first).
pub fn load_notes(project_id: &str) -> Result<Vec<Note>, String> {
    let path = notes_file(project_id);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let f = fs::File::open(&path).map_err(|e| format!("read notes: {e}"))?;
    let reader = BufReader::new(f);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| format!("read line: {e}"))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<Note>(line) {
            Ok(n) => out.push(n),
            Err(_) => continue, // skip corrupt lines
        }
    }
    Ok(out)
}

/// Search notes with optional filters; returns newest-first.
pub fn search_notes(args: NoteSearchArgs) -> Result<Vec<Note>, String> {
    let mut notes = load_notes(&args.project_id)?;
    let limit = args.limit.unwrap_or(20).clamp(1, 100);
    let q = args
        .query
        .as_deref()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty());
    let kind = args.kind.as_deref().map(NoteKind::parse);

    notes.retain(|n| {
        if let Some(ref k) = kind {
            if &n.kind != k {
                return false;
            }
        }
        if let Some(sid) = args.shot_id.as_deref() {
            if n.shot_id.as_deref() != Some(sid) {
                return false;
            }
        }
        if let Some(sc) = args.scene_id.as_deref() {
            if n.scene_id.as_deref() != Some(sc) {
                return false;
            }
        }
        if let Some(ref q) = q {
            let hay = format!(
                "{} {} {} {}",
                n.title,
                n.body,
                n.tags.join(" "),
                n.kind.as_str()
            )
            .to_ascii_lowercase();
            if !hay.contains(q) {
                return false;
            }
        }
        true
    });

    notes.reverse(); // newest first (file is append-oldest-first)
    notes.truncate(limit);
    Ok(notes)
}

/// Compact text block for injecting into First AD / factory prompts.
pub fn notes_prompt_block(project_id: &str, limit: usize) -> String {
    let notes = search_notes(NoteSearchArgs {
        project_id: project_id.to_string(),
        query: None,
        kind: None,
        shot_id: None,
        scene_id: None,
        limit: Some(limit),
    })
    .unwrap_or_default();
    if notes.is_empty() {
        return String::new();
    }
    let mut lines = vec!["ATOMIC NOTES (recent memory):".to_string()];
    for n in notes {
        lines.push(format!(
            "- [{}] {} — {}",
            n.kind.as_str(),
            n.title,
            n.body.chars().take(200).collect::<String>()
        ));
    }
    lines.join("\n")
}

/// Convenience writers used by factory / First AD.
pub fn note_quality(
    project_id: &str,
    scene_id: Option<&str>,
    shot_id: Option<&str>,
    shot_name: &str,
    overall: f64,
    accept: bool,
    summary: &str,
    issues: &[String],
) -> Result<Note, String> {
    write_note(NoteWriteArgs {
        project_id: project_id.to_string(),
        kind: "quality_feedback".into(),
        title: format!(
            "{} — {}",
            shot_name,
            if accept { "PASS" } else { "FAIL" }
        ),
        body: format!(
            "overall={overall:.2} accept={accept}. {summary}. issues: {}",
            if issues.is_empty() {
                "none".into()
            } else {
                issues.join("; ")
            }
        ),
        tags: vec!["quality".into(), if accept { "pass" } else { "fail" }.into()],
        scene_id: scene_id.map(|s| s.to_string()),
        shot_id: shot_id.map(|s| s.to_string()),
    })
}

pub fn note_continuity(
    project_id: &str,
    scene_id: Option<&str>,
    title: &str,
    body: &str,
) -> Result<Note, String> {
    write_note(NoteWriteArgs {
        project_id: project_id.to_string(),
        kind: "continuity".into(),
        title: title.to_string(),
        body: body.to_string(),
        tags: vec!["continuity".into()],
        scene_id: scene_id.map(|s| s.to_string()),
        shot_id: None,
    })
}

pub fn note_scene_plan(
    project_id: &str,
    scene_id: Option<&str>,
    summary: &str,
    detail: &str,
) -> Result<Note, String> {
    write_note(NoteWriteArgs {
        project_id: project_id.to_string(),
        kind: "scene_plan".into(),
        title: summary.chars().take(80).collect(),
        body: detail.to_string(),
        tags: vec!["first_ad".into(), "plan".into()],
        scene_id: scene_id.map(|s| s.to_string()),
        shot_id: None,
    })
}

/// Path helpers for tests.
pub fn notes_path_for(project_id: &str) -> PathBuf {
    notes_file(project_id)
}

pub fn notes_dir_exists(path: &Path) -> bool {
    path.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_project_id() -> String {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("note-test-{n}-{}", std::process::id())
    }

    #[test]
    fn write_and_search_roundtrip() {
        let prev = std::env::var("SLATE_DATA_DIR").ok();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("SLATE_DATA_DIR", dir.path());

        let pid = temp_project_id();
        // ensure project folder exists for notes parent
        fs::create_dir_all(projects_root().join(&pid)).unwrap();

        let n = write_note(NoteWriteArgs {
            project_id: pid.clone(),
            kind: "continuity".into(),
            title: "Amber jacket".into(),
            body: "Kaia wears amber leather jacket always".into(),
            tags: vec!["wardrobe".into()],
            scene_id: Some("sc1".into()),
            shot_id: None,
        })
        .unwrap();
        assert!(!n.id.is_empty());

        let hits = search_notes(NoteSearchArgs {
            project_id: pid.clone(),
            query: Some("jacket".into()),
            kind: Some("continuity".into()),
            shot_id: None,
            scene_id: None,
            limit: Some(10),
        })
        .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Amber jacket");

        let block = notes_prompt_block(&pid, 5);
        assert!(block.contains("ATOMIC NOTES"));
        assert!(block.contains("Amber jacket"));

        if let Some(p) = prev {
            std::env::set_var("SLATE_DATA_DIR", p);
        } else {
            std::env::remove_var("SLATE_DATA_DIR");
        }
    }
}
