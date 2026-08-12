//! Pack manifest — maps logical generation fields onto ComfyUI API graph nodes.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Logical input → graph node field mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputMap {
    pub node_id: String,
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default)]
    pub optional: bool,
}

/// Output media slot in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputMap {
    pub node_id: String,
    #[serde(rename = "type")]
    pub kind: String,
}

/// Pack capability limits (aspects, duration, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackLimits {
    #[serde(default)]
    pub aspect_ratios: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_duration_sec: Option<f64>,
}

/// Checked-in pack descriptor: id, injection map, outputs, limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackManifest {
    pub id: String,
    pub label: String,
    pub modality: String,
    #[serde(default)]
    pub inputs: HashMap<String, InputMap>,
    #[serde(default)]
    pub outputs: HashMap<String, OutputMap>,
    #[serde(default)]
    pub limits: PackLimits,
    pub compile_profile: String,
}

impl Default for PackLimits {
    fn default() -> Self {
        Self {
            aspect_ratios: Vec::new(),
            max_duration_sec: None,
        }
    }
}

/// Load and deserialize a pack `manifest.json` from disk.
pub fn load_manifest(path: &Path) -> Result<PackManifest> {
    let text = fs::read_to_string(path).map_err(|e| Error::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    let manifest: PackManifest = serde_json::from_str(&text)?;
    Ok(manifest)
}
