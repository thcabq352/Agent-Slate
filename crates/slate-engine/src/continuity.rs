//! Scene-level continuity context for First AD orchestration (Phase 3).
//!
//! Accumulates wardrobe, weather, handoff, and quality notes across shots so
//! generate/judge receive structured continuity, not only the static bible.

use serde::{Deserialize, Serialize};
use slate_brain::QualityVerdict;
use slate_domain::Project;

/// One completed beat in the scene run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuityBeat {
    pub shot_id: String,
    pub shot_name: String,
    pub intent: String,
    /// Path of last accepted (or final) take for this shot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_take_path: Option<String>,
    /// Short visual handoff for the next shot ("opens where shot 01 ended: …").
    #[serde(default)]
    pub handoff: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_overall: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_accepted: Option<bool>,
}

/// Running continuity book for one scene (in-memory during factory / First AD).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneContinuityContext {
    pub project_id: String,
    pub scene_id: String,
    pub scene_name: String,
    /// Free-form locks from First AD / bible (wardrobe, weather, time of day).
    #[serde(default)]
    pub locks: Vec<String>,
    /// Ordered completed shots.
    #[serde(default)]
    pub beats: Vec<ContinuityBeat>,
    /// Rolling notes the AD wants every later shot to honor.
    #[serde(default)]
    pub standing_orders: Vec<String>,
}

impl SceneContinuityContext {
    pub fn from_project_scene(project: &Project, scene_idx: usize) -> Self {
        let scene = project.scenes.get(scene_idx);
        let mut locks = Vec::new();
        if !project.world.is_empty() {
            locks.push(format!("World/tone: {}", project.world));
        }
        for c in &project.characters {
            let cloth = if c.clothing.is_empty() {
                String::new()
            } else {
                format!(" clothing={}", c.clothing)
            };
            locks.push(format!("Cast {}{}", c.name, cloth));
        }
        for loc in &project.locations {
            let weather = if loc.weather.is_empty() {
                String::new()
            } else {
                format!(" weather={}", loc.weather)
            };
            let tod = if loc.time_of_day.is_empty() {
                String::new()
            } else {
                format!(" time={}", loc.time_of_day)
            };
            locks.push(format!(
                "Location {}{}{}",
                loc.name, weather, tod
            ));
        }

        Self {
            project_id: project.id.clone(),
            scene_id: scene.map(|s| s.id.clone()).unwrap_or_default(),
            scene_name: scene.map(|s| s.name.clone()).unwrap_or_default(),
            locks,
            beats: Vec::new(),
            standing_orders: Vec::new(),
        }
    }

    /// Record a finished generate/judge cycle for a shot.
    pub fn record_shot(
        &mut self,
        shot_id: &str,
        shot_name: &str,
        intent: &str,
        take_path: Option<&str>,
        quality: Option<&QualityVerdict>,
        handoff: impl Into<String>,
    ) {
        let handoff = handoff.into();
        if let Some(q) = quality {
            if !q.accept {
                for issue in q.issues.iter().take(3) {
                    let note = format!("Open issue after {shot_name}: {issue}");
                    if !self.standing_orders.iter().any(|s| s == &note) {
                        self.standing_orders.push(note);
                    }
                }
            } else if q.overall >= 0.85 {
                // Promote a positive lock when judge is strong.
                let lock = format!(
                    "Locked look from {shot_name} (score {:.2}): keep wardrobe/light consistent",
                    q.overall
                );
                if !self.locks.iter().any(|s| s == &lock) {
                    self.locks.push(lock);
                }
            }
        }

        self.beats.push(ContinuityBeat {
            shot_id: shot_id.to_string(),
            shot_name: shot_name.to_string(),
            intent: intent.to_string(),
            last_take_path: take_path.map(|s| s.to_string()),
            handoff,
            quality_overall: quality.map(|q| q.overall),
            quality_accepted: quality.map(|q| q.accept),
        });
    }

    /// Text block for prompts / VL judge / First AD.
    pub fn as_prompt_block(&self) -> String {
        let mut lines = vec![
            format!("SCENE CONTINUITY — {} [{}]", self.scene_name, self.scene_id),
        ];
        if !self.locks.is_empty() {
            lines.push("LOCKS:".into());
            for l in &self.locks {
                lines.push(format!("- {l}"));
            }
        }
        if !self.standing_orders.is_empty() {
            lines.push("STANDING ORDERS:".into());
            for o in self.standing_orders.iter().rev().take(8).collect::<Vec<_>>().into_iter().rev() {
                lines.push(format!("- {o}"));
            }
        }
        if !self.beats.is_empty() {
            lines.push("BEATS SO FAR:".into());
            for b in &self.beats {
                let q = match (b.quality_accepted, b.quality_overall) {
                    (Some(true), Some(s)) => format!(" PASS {s:.2}"),
                    (Some(false), Some(s)) => format!(" FAIL {s:.2}"),
                    _ => String::new(),
                };
                lines.push(format!(
                    "- {} ({}) intent={}{} handoff={}",
                    b.shot_name, b.shot_id, b.intent, q, b.handoff
                ));
            }
            if let Some(last) = self.beats.last() {
                lines.push(format!(
                    "HANDOFF INTO NEXT SHOT: open exactly where \"{}\" left: {}",
                    last.shot_name,
                    if last.handoff.is_empty() {
                        last.intent.as_str()
                    } else {
                        last.handoff.as_str()
                    }
                ));
            }
        }
        lines.join("\n")
    }

    pub fn summary_one_line(&self) -> String {
        format!(
            "scene={} beats={} locks={} orders={}",
            self.scene_name,
            self.beats.len(),
            self.locks.len(),
            self.standing_orders.len()
        )
    }
}

/// Static bible context + optional live continuity book.
pub fn full_continuity_text(
    project: &Project,
    scene_idx: usize,
    shot_idx: usize,
    live: Option<&SceneContinuityContext>,
) -> String {
    let mut base = crate::factory::continuity_context(project, scene_idx, shot_idx);
    if let Some(live) = live {
        base.push_str("\n\n");
        base.push_str(&live.as_prompt_block());
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;
    use slate_brain::{QualityScores, QualityVerdict};
    use slate_domain::new_project;

    fn sample_verdict(overall: f64, accept: bool) -> QualityVerdict {
        let s = QualityScores {
            visual_quality: overall,
            continuity: overall,
            artifacts: overall,
            prompt_fidelity: overall,
        };
        let mut v = QualityVerdict::from_scores(s, 0.7, vec![], vec![], "ok".into());
        v.accept = accept;
        v
    }

    #[test]
    fn accumulates_beats_and_handoff() {
        let mut p = new_project("T");
        p.scenes.push(slate_domain::Scene {
            id: "sc1".into(),
            name: "Rooftop".into(),
            synopsis: "Chase".into(),
            shots: vec![],
        });
        let mut ctx = SceneContinuityContext::from_project_scene(&p, 0);
        assert_eq!(ctx.scene_name, "Rooftop");

        let q = sample_verdict(0.9, true);
        ctx.record_shot(
            "sh1",
            "Shot 01",
            "sprint",
            Some("/tmp/a.png"),
            Some(&q),
            "ends on roof edge looking down alley",
        );
        assert_eq!(ctx.beats.len(), 1);
        let block = ctx.as_prompt_block();
        assert!(block.contains("HANDOFF INTO NEXT SHOT"));
        assert!(block.contains("roof edge"));
    }
}
