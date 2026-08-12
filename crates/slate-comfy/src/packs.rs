//! Discover installed Comfy packs under `workflows/packs`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::manifest::load_manifest;
use crate::Result;

/// Summary of one pack directory.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub id: String,
    pub label: String,
    pub modality: String,
    pub path: String,
    pub has_workflow: bool,
    pub ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// List pack folders that contain `manifest.json`.
pub fn list_packs(packs_dir: &Path) -> Result<Vec<PackInfo>> {
    let mut out = Vec::new();
    if !packs_dir.is_dir() {
        return Ok(out);
    }
    let rd = fs::read_dir(packs_dir).map_err(|e| crate::Error::Io {
        path: packs_dir.display().to_string(),
        source: e,
    })?;
    for ent in rd.flatten() {
        if !ent.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let dir = ent.path();
        let man = dir.join("manifest.json");
        if !man.is_file() {
            continue;
        }
        let wf = dir.join("workflow.api.json");
        let has_workflow = wf.is_file();
        match load_manifest(&man) {
            Ok(m) => {
                let placeholder = has_workflow
                    && fs::read_to_string(&wf)
                        .map(|t| t.contains("PLACEHOLDER") || t.contains("ALIGN_ME"))
                        .unwrap_or(false);
                let ready = has_workflow && !placeholder;
                out.push(PackInfo {
                    id: m.id,
                    label: m.label,
                    modality: m.modality,
                    path: dir.display().to_string(),
                    has_workflow,
                    ready,
                    note: if placeholder {
                        Some(
                            "template graph — align node ids / checkpoints before live generate"
                                .into(),
                        )
                    } else {
                        None
                    },
                });
            }
            Err(e) => {
                out.push(PackInfo {
                    id: ent.file_name().to_string_lossy().into(),
                    label: ent.file_name().to_string_lossy().into(),
                    modality: "unknown".into(),
                    path: dir.display().to_string(),
                    has_workflow,
                    ready: false,
                    note: Some(format!("manifest error: {e}")),
                });
            }
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// Resolve packs dir helper for tests.
pub fn packs_dir_from_workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../workflows/packs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_default_still() {
        let dir = packs_dir_from_workspace();
        let packs = list_packs(&dir).unwrap();
        assert!(
            packs.iter().any(|p| p.id == "default-still" && p.ready),
            "expected default-still ready, got {packs:?}"
        );
        assert!(
            packs.iter().any(|p| p.id == "default-video" && p.ready),
            "expected default-video ready (LTX graph, no PLACEHOLDER), got {packs:?}"
        );
    }
}
