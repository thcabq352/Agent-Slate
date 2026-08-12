//! Inject logical values into a ComfyUI API-format workflow graph.

use std::collections::HashMap;

use rand::Rng;
use serde_json::{json, Value};

use crate::manifest::{InputMap, PackManifest};
use crate::{Error, Result};

/// Apply manifest-mapped values onto a cloned workflow graph.
///
/// For each logical key in `values`, writes `workflow[node_id].inputs[field] = value`.
/// Inputs with `mode: "randomize"` and no provided value get a random `u64` (seed).
/// Missing required (non-optional) inputs error; optional missing inputs are skipped.
pub fn inject_workflow(
    mut workflow: Value,
    manifest: &PackManifest,
    values: &HashMap<String, Value>,
) -> Result<Value> {
    let graph = workflow
        .as_object_mut()
        .ok_or_else(|| Error::Inject("workflow root must be a JSON object".into()))?;

    for (logical, map) in &manifest.inputs {
        let value = match values.get(logical) {
            Some(v) => v.clone(),
            None if is_randomize(map) => json!(random_seed()),
            None if map.optional => continue,
            None => {
                return Err(Error::Inject(format!(
                    "missing required input `{logical}` (node {}, field {})",
                    map.node_id, map.field
                )));
            }
        };
        set_input(graph, &map.node_id, &map.field, value.clone())?;
        for m in &map.mirrors {
            set_input(graph, &m.node_id, &m.field, value.clone())?;
        }
    }

    Ok(workflow)
}

fn is_randomize(map: &InputMap) -> bool {
    map.mode
        .as_deref()
        .map(|m| m.eq_ignore_ascii_case("randomize"))
        .unwrap_or(false)
}

fn random_seed() -> u64 {
    rand::thread_rng().gen()
}

fn set_input(
    graph: &mut serde_json::Map<String, Value>,
    node_id: &str,
    field: &str,
    value: Value,
) -> Result<()> {
    let node = graph.get_mut(node_id).ok_or_else(|| {
        Error::Inject(format!(
            "node `{node_id}` not found in workflow (field {field})"
        ))
    })?;
    let node_obj = node
        .as_object_mut()
        .ok_or_else(|| Error::Inject(format!("node `{node_id}` is not a JSON object")))?;
    let inputs = node_obj
        .entry("inputs")
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    let inputs_obj = inputs.as_object_mut().ok_or_else(|| {
        Error::Inject(format!("node `{node_id}` inputs is not a JSON object"))
    })?;
    inputs_obj.insert(field.to_string(), value);
    Ok(())
}
