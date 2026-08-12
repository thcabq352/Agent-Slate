//! Music cue compile — text deliverables for Suno / Eleven / generic models.
//! Generation via Comfy is not bundled; this produces copy-ready prompts.

use serde::{Deserialize, Serialize};
use slate_domain::{open_project, MusicCue, VocalsPreference};

/// One compiled music deliverable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompiledMusic {
    pub cue_id: String,
    pub name: String,
    pub target: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyrics: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_sec: Option<f64>,
}

fn vocals_word(v: &VocalsPreference) -> &'static str {
    match v {
        VocalsPreference::Instrumental => "instrumental, no vocals",
        VocalsPreference::Vocals => "with vocals",
        VocalsPreference::Either => "instrumental or vocals",
    }
}

/// Rule-based compile (no LLM) — always available.
pub fn compile_cue(cue: &MusicCue, target: &str) -> CompiledMusic {
    let mut parts = Vec::new();
    if !cue.intent.is_empty() {
        parts.push(cue.intent.clone());
    }
    if !cue.genre.is_empty() {
        parts.push(cue.genre.clone());
    }
    if !cue.mood.is_empty() {
        parts.push(format!("{} mood", cue.mood));
    }
    if !cue.tempo.is_empty() {
        parts.push(cue.tempo.clone());
    }
    if !cue.instrumentation.is_empty() {
        parts.push(cue.instrumentation.clone());
    }
    if !cue.era.is_empty() {
        parts.push(cue.era.clone());
    }
    if !cue.structure.is_empty() {
        parts.push(format!("structure: {}", cue.structure));
    }
    parts.push(vocals_word(&cue.vocals).to_string());
    if !cue.lyric_theme.is_empty() && !matches!(cue.vocals, VocalsPreference::Instrumental) {
        parts.push(format!("lyric theme: {}", cue.lyric_theme));
    }
    if let Some(d) = cue.duration_sec {
        parts.push(format!("{d:.0} seconds"));
    }
    if !cue.notes.is_empty() {
        parts.push(cue.notes.clone());
    }

    let prompt = match target {
        "suno" | "suno-v4" => format!(
            "[Style: {}] {}\n{}",
            [cue.genre.as_str(), cue.mood.as_str(), cue.era.as_str()]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(", "),
            parts.join(". "),
            if cue.lyrics.is_empty() {
                String::new()
            } else {
                format!("\n{}", cue.lyrics)
            }
        )
        .trim()
        .to_string(),
        _ => parts.join(". "),
    };

    CompiledMusic {
        cue_id: cue.id.clone(),
        name: cue.name.clone(),
        target: target.to_string(),
        prompt,
        lyrics: if cue.lyrics.is_empty() {
            None
        } else {
            Some(cue.lyrics.clone())
        },
        duration_sec: cue.duration_sec,
    }
}

/// Compile one cue or all cues in a project.
pub fn compile_project_music(
    project_id: &str,
    cue_id: Option<&str>,
    target: &str,
) -> Result<Vec<CompiledMusic>, String> {
    let p = open_project(project_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;
    let cues = p.music.unwrap_or_default();
    if cues.is_empty() {
        return Ok(Vec::new());
    }
    let selected: Vec<&MusicCue> = if let Some(id) = cue_id {
        cues.iter()
            .filter(|c| c.id == id || c.name.eq_ignore_ascii_case(id))
            .collect()
    } else {
        cues.iter().collect()
    };
    if selected.is_empty() {
        return Err(format!("music cue not found: {cue_id:?}"));
    }
    Ok(selected.iter().map(|c| compile_cue(c, target)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use slate_domain::VocalsPreference;

    fn sample_cue() -> MusicCue {
        MusicCue {
            id: "c1".into(),
            name: "Rooftop pulse".into(),
            scene_ref: "sc1".into(),
            intent: "build tension".into(),
            genre: "synthwave".into(),
            mood: "urgent".into(),
            tempo: "128 BPM".into(),
            instrumentation: "analog bass, drums".into(),
            era: "1980s".into(),
            structure: "sparse to full kit".into(),
            vocals: VocalsPreference::Instrumental,
            lyric_theme: String::new(),
            lyrics: String::new(),
            duration_sec: Some(24.0),
            notes: String::new(),
        }
    }

    #[test]
    fn compile_includes_genre_and_duration() {
        let c = compile_cue(&sample_cue(), "generic");
        assert!(c.prompt.contains("synthwave"));
        assert!(c.prompt.contains("24"));
        assert!(c.prompt.contains("instrumental"));
    }
}
