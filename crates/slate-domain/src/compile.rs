//! Rule-based ComfyUI compile — sectioned prompt → positive/negative + size.
//! V1 is pure string transform; no LLM.

use std::collections::HashSet;

use crate::types::Shot;

/// Fixed quality baseline for V1 stills negative prompt.
pub const DEFAULT_NEGATIVE: &str = "blurry, low quality, watermark, text overlay, deformed hands";

/// Compiled payload ready for a ComfyUI stills workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledPrompt {
    pub positive: String,
    pub negative: String,
    pub width: u32,
    pub height: u32,
}

/// Map aspect ratio string to pixel dimensions. Unknown → 16:9 default.
pub fn aspect_size(aspect: &str) -> (u32, u32) {
    match aspect.trim() {
        "16:9" => (1280, 720),
        "9:16" => (720, 1280),
        "1:1" => (1024, 1024),
        _ => (1280, 720),
    }
}

/// Markdown section header (`# Subject`, `## Composition`, …).
fn is_section_header(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with('#')
}

/// Compile a shot into Comfy-ready positive/negative prompts and canvas size.
///
/// Positive: strip `# Section` headers, skip muted 1-based line indices, join
/// non-empty body lines with spaces.
/// Negative: fixed V1 quality baseline.
pub fn compile_for_comfy(shot: &Shot, aspect: &str) -> CompiledPrompt {
    let (width, height) = aspect_size(aspect);
    let muted: HashSet<u32> = shot.muted_lines.iter().copied().collect();

    let positive = shot
        .prompt
        .lines()
        .enumerate()
        .filter(|(i, line)| {
            let line_no = (*i as u32) + 1; // 1-based, matches locked/muted convention
            if muted.contains(&line_no) {
                return false;
            }
            let t = line.trim();
            if t.is_empty() {
                return false;
            }
            !is_section_header(t)
        })
        .map(|(_, line)| line.trim())
        .collect::<Vec<_>>()
        .join(" ");

    CompiledPrompt {
        positive,
        negative: DEFAULT_NEGATIVE.to_string(),
        width,
        height,
    }
}
