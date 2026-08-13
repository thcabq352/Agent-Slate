//! Project persistence — one folder per project under Documents/Slate (or SLATE_DATA_DIR).
//! Mirrors `src/main/projects.ts`: `project.json` + atomic temp+rename writes.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::types::{new_project, Project, TakeRating};

/// Lightweight listing row for the home screen / navigator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMeta {
    pub id: String,
    pub name: String,
    pub logline: String,
    pub path: String,
    pub updated_at: String,
    pub scene_count: usize,
    pub shot_count: usize,
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Root directory for project folders.
/// Override with `SLATE_DATA_DIR` (portable installs, tests).
pub fn projects_root() -> PathBuf {
    if let Ok(dir) = env::var("SLATE_DATA_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    dirs::document_dir()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("Slate")
}

fn project_dir(id: &str) -> PathBuf {
    projects_root().join(id)
}

fn project_file(id: &str) -> PathBuf {
    project_dir(id).join("project.json")
}

fn ensure_root() -> Result<()> {
    fs::create_dir_all(projects_root())?;
    Ok(())
}

/// List all readable projects under the root, newest `updated_at` first.
/// Unreadable dirs / bad JSON are skipped (never crash the list).
pub fn list_projects() -> Result<Vec<ProjectMeta>> {
    ensure_root()?;
    let root = projects_root();
    let mut metas = Vec::new();

    let entries = match fs::read_dir(&root) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(metas),
        Err(e) => return Err(e.into()),
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if !file_type.is_dir() {
            continue;
        }
        let id_name = entry.file_name();
        let id_str = id_name.to_string_lossy();
        let file = project_file(&id_str);
        if !file.is_file() {
            continue;
        }
        match load_project_from_path(&file) {
            Ok(p) => {
                let shot_count = p.scenes.iter().map(|s| s.shots.len()).sum();
                metas.push(ProjectMeta {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    logline: p.logline.clone(),
                    path: project_dir(&p.id).to_string_lossy().into_owned(),
                    updated_at: p.updated_at.clone(),
                    scene_count: p.scenes.len(),
                    shot_count,
                });
            }
            Err(_) => {
                // Unreadable project file — skip rather than crash the list.
            }
        }
    }

    metas.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(metas)
}

/// Create a new empty project on disk and return it.
pub fn create_project(name: &str) -> Result<Project> {
    ensure_root()?;
    let mut p = new_project(name);
    fs::create_dir_all(project_dir(&p.id))?;
    save_project(&mut p)?;
    Ok(p)
}

/// Open a project by id. Returns `Ok(None)` if missing or unreadable (mirrors TS).
pub fn open_project(id: &str) -> Result<Option<Project>> {
    let file = project_file(id);
    match load_project_from_path(&file) {
        Ok(p) => Ok(Some(p)),
        Err(e) => {
            if let Some(ioe) = e.downcast_ref::<io::Error>() {
                if ioe.kind() == io::ErrorKind::NotFound {
                    return Ok(None);
                }
            }
            // Parse / other read failures: treat as missing like TS catch → null.
            // But if it's a genuine unexpected error type, still return None to match TS.
            let _ = e;
            Ok(None)
        }
    }
}

/// Persist `project`, bumping `updated_at`. Atomic write via temp file + rename.
pub fn save_project(project: &mut Project) -> Result<()> {
    project.updated_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let dir = project_dir(&project.id);
    fs::create_dir_all(&dir)?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let tmp = dir.join(format!(".project.{ts}.tmp"));
    let dest = project_file(&project.id);

    let json = serde_json::to_string_pretty(project)?;
    fs::write(&tmp, json.as_bytes())?;
    // On Windows, rename over existing dest can fail; remove dest first if present.
    if dest.exists() {
        fs::remove_file(&dest)?;
    }
    fs::rename(&tmp, &dest)?;
    Ok(())
}

/// Result of circling a take (persisted to project.json).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CircledTakeResult {
    pub ok: bool,
    pub project_id: String,
    pub scene_id: String,
    pub shot_id: String,
    pub shot_name: String,
    pub take_id: String,
    pub rating: TakeRating,
}

/// Mark a take as circled. `take_id` wins; else latest take on `shot_id`; else latest in the project.
pub fn circle_take(
    project_id: &str,
    take_id: Option<&str>,
    shot_id: Option<&str>,
) -> Result<CircledTakeResult> {
    let mut project = open_project(project_id)?.ok_or("Project not found")?;

    let mut found: Option<(usize, usize, usize)> = None;
    if let Some(tid) = take_id.filter(|s| !s.is_empty()) {
        for (si, scene) in project.scenes.iter().enumerate() {
            for (shi, shot) in scene.shots.iter().enumerate() {
                if let Some(ti) = shot.takes.iter().position(|t| t.id == tid) {
                    found = Some((si, shi, ti));
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        if found.is_none() {
            return Err(format!("take not found: {tid}").into());
        }
    } else if let Some(sid) = shot_id.filter(|s| !s.is_empty()) {
        for (si, scene) in project.scenes.iter().enumerate() {
            if let Some(shi) = scene
                .shots
                .iter()
                .position(|s| s.id == sid || s.name == sid)
            {
                let shot = &scene.shots[shi];
                if shot.takes.is_empty() {
                    return Err(format!("no takes on shot {sid}").into());
                }
                found = Some((si, shi, shot.takes.len() - 1));
                break;
            }
        }
        if found.is_none() {
            return Err(format!("shot not found: {sid}").into());
        }
    } else {
        let mut best: Option<(usize, usize, usize, String)> = None;
        for (si, scene) in project.scenes.iter().enumerate() {
            for (shi, shot) in scene.shots.iter().enumerate() {
                for (ti, take) in shot.takes.iter().enumerate() {
                    let ts = take.logged_at.clone();
                    let better = best
                        .as_ref()
                        .map(|b| ts.as_str() >= b.3.as_str())
                        .unwrap_or(true);
                    if better {
                        best = Some((si, shi, ti, ts));
                    }
                }
            }
        }
        found = best.map(|(si, shi, ti, _)| (si, shi, ti));
        if found.is_none() {
            return Err("no takes on this project".into());
        }
    }

    let (si, shi, ti) = found.unwrap();
    {
        let take = &mut project.scenes[si].shots[shi].takes[ti];
        take.rating = TakeRating::Circled;
        if !take.notes.to_ascii_lowercase().contains("human approved") {
            if take.notes.trim().is_empty() {
                take.notes = "human approved".into();
            } else {
                take.notes = format!("{} | human approved", take.notes.trim());
            }
        }
    }
    let scene_id = project.scenes[si].id.clone();
    let shot_id_out = project.scenes[si].shots[shi].id.clone();
    let shot_name = project.scenes[si].shots[shi].name.clone();
    let take_id_out = project.scenes[si].shots[shi].takes[ti].id.clone();
    save_project(&mut project)?;
    Ok(CircledTakeResult {
        ok: true,
        project_id: project.id,
        scene_id,
        shot_id: shot_id_out,
        shot_name,
        take_id: take_id_out,
        rating: TakeRating::Circled,
    })
}

fn load_project_from_path(path: &Path) -> Result<Project> {
    let data = fs::read_to_string(path)?;
    let p: Project = serde_json::from_str(&data)?;
    Ok(p)
}
