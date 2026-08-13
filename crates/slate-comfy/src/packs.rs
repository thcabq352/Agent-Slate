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

/// True when `dir` looks like `workflows/packs` (has a known pack manifest).
pub fn is_packs_dir(dir: &Path) -> bool {
    dir.join("default-still").join("manifest.json").is_file()
        || dir.join("default-video").join("manifest.json").is_file()
}

/// Locate Comfy packs relative to the **engine binary**, not process cwd.
///
/// Order: `SLATE_PACKS_DIR` (if set) → walk up from `current_exe` looking for
/// `workflows/packs` → cwd last (compat only).
pub fn resolve_packs_dir() -> PathBuf {
    resolve_packs_dir_from(
        std::env::var("SLATE_PACKS_DIR").ok(),
        std::env::current_exe().ok(),
        std::env::current_dir().ok(),
    )
}

/// Testable resolver. `env_override` wins when non-empty.
pub fn resolve_packs_dir_from(
    env_override: Option<String>,
    current_exe: Option<PathBuf>,
    cwd: Option<PathBuf>,
) -> PathBuf {
    if let Some(p) = env_override.filter(|s| !s.is_empty()) {
        let pb = PathBuf::from(p);
        return pb.canonicalize().unwrap_or(pb);
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(exe) = current_exe {
        if let Some(mut dir) = exe.parent().map(Path::to_path_buf) {
            for _ in 0..8 {
                candidates.push(dir.join("workflows").join("packs"));
                candidates.push(dir.join("packs"));
                if !dir.pop() {
                    break;
                }
            }
        }
    }
    if let Some(cwd) = cwd {
        candidates.push(cwd.join("workflows").join("packs"));
        candidates.push(cwd.join("packs"));
    }

    for c in &candidates {
        if is_packs_dir(c) {
            return c.canonicalize().unwrap_or_else(|_| c.clone());
        }
    }

    candidates
        .into_iter()
        .next()
        .unwrap_or_else(|| PathBuf::from("workflows/packs"))
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
        assert!(
            packs.iter().any(|p| p.id == "default-i2v" && p.ready),
            "expected default-i2v ready, got {packs:?}"
        );
        assert!(
            packs.iter().any(|p| p.id == "default-flf2v" && p.ready),
            "expected default-flf2v ready, got {packs:?}"
        );
    }

    #[test]
    fn resolve_packs_prefers_exe_tree_over_cwd() {
        let workspace = packs_dir_from_workspace()
            .canonicalize()
            .expect("workspace packs");
        assert!(is_packs_dir(&workspace), "fixture packs missing: {workspace:?}");
        // Fake exe under target/debug — file need not exist; we walk parents.
        let repo = workspace
            .parent()
            .and_then(|p| p.parent())
            .expect("workflows/packs → repo");
        let exe = repo.join("target").join("debug").join("slate-engine.exe");
        let bogus_cwd = PathBuf::from("/definitely/not/the/repo");
        let resolved = resolve_packs_dir_from(None, Some(exe), Some(bogus_cwd));
        assert!(
            is_packs_dir(&resolved),
            "expected packs next to binary walk, got {}",
            resolved.display()
        );
    }

    #[test]
    fn resolve_packs_env_override_wins() {
        let workspace = packs_dir_from_workspace();
        let resolved = resolve_packs_dir_from(
            Some(workspace.display().to_string()),
            Some(PathBuf::from("/tmp/slate-engine")),
            Some(PathBuf::from("/")),
        );
        assert!(is_packs_dir(&resolved), "env override should find packs");
    }
}
