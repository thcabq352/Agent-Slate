//! Unit tests for history output parsing (no live Comfy required).

use slate_comfy::{collect_output_files, collect_output_files_preferring};

#[test]
fn collect_output_files_from_history_shape() {
    let history = serde_json::json!({
        "outputs": {
            "9": {
                "images": [{ "filename": "a.png", "subfolder": "", "type": "output" }]
            }
        }
    });
    let files = collect_output_files(&history);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].filename, "a.png");
    assert_eq!(files[0].subfolder, "");
    assert_eq!(files[0].file_type, "output");
}

#[test]
fn collect_output_files_empty_when_no_outputs() {
    let history = serde_json::json!({ "status": { "completed": true } });
    assert!(collect_output_files(&history).is_empty());
}

#[test]
fn collect_output_files_multiple_nodes() {
    let history = serde_json::json!({
        "outputs": {
            "9": {
                "images": [
                    { "filename": "a.png", "subfolder": "", "type": "output" }
                ]
            },
            "10": {
                "images": [
                    { "filename": "b.png", "subfolder": "shots", "type": "output" }
                ]
            }
        }
    });
    let files = collect_output_files(&history);
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.filename == "a.png"));
    assert!(files
        .iter()
        .any(|f| f.filename == "b.png" && f.subfolder == "shots"));
}

#[test]
fn collect_output_files_savevideo_uses_images_key() {
    // Comfy SaveVideo / PreviewVideo serializes as images + animated.
    let history = serde_json::json!({
        "outputs": {
            "90": {
                "images": [{
                    "filename": "slate_video_00001_.mp4",
                    "subfolder": "",
                    "type": "output"
                }],
                "animated": [true]
            }
        }
    });
    let files = collect_output_files(&history);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].filename, "slate_video_00001_.mp4");
}

#[test]
fn prefers_declared_output_node_over_preview() {
    let history = serde_json::json!({
        "outputs": {
            "8": {
                "images": [{ "filename": "preview.png", "subfolder": "", "type": "temp" }]
            },
            "90": {
                "images": [{
                    "filename": "slate_video_00001_.mp4",
                    "subfolder": "",
                    "type": "output"
                }]
            }
        }
    });
    let all = collect_output_files(&history);
    assert_eq!(all.len(), 2);
    let media = collect_output_files_preferring(&history, Some("90"));
    assert_eq!(media.len(), 1);
    assert_eq!(media[0].filename, "slate_video_00001_.mp4");
    assert_eq!(media[0].file_type, "output");
}
