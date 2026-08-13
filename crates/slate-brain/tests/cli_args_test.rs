use slate_brain::{
    build_cursor_args, build_grok_build_args, grok_auth_looks_signed_in, grok_build_cli_model,
    parse_cursor_output, parse_grok_build_output, BrainBackend, BrainRequest, BrainTier,
};
use std::path::Path;

fn sample_req(tier: BrainTier) -> BrainRequest {
    BrainRequest {
        id: "1".into(),
        task: "t".into(),
        system: "sys".into(),
        prompt: "hello".into(),
        images: vec![],
        tier,
        expect_json: false,
        local_endpoint: None,
        local_model: None,
    }
}

#[test]
fn cursor_args_include_print_json_ask_and_composer() {
    let req = sample_req(BrainTier::Fast);
    let ws = Path::new("/tmp/slate-cursor-brain");
    let (args, input) = build_cursor_args(&req, ws, BrainBackend::Cursor);
    assert!(args.iter().any(|a| a == "-p"));
    assert!(args.windows(2).any(|w| w == ["--output-format", "json"]));
    assert!(args.windows(2).any(|w| w == ["--mode", "ask"]));
    assert!(args.iter().any(|a| a == "--trust"));
    assert!(args.windows(2).any(|w| w == ["--model", "composer-2.5-fast"]));
    assert!(args.windows(2).any(|w| w == ["--workspace", ws.to_str().unwrap()]));
    assert!(input.contains("sys"));
    assert!(input.contains("hello"));
}

#[test]
fn grok_45_and_46_model_slugs() {
    let ws = Path::new("/tmp/slate-cursor-brain");
    let fast = sample_req(BrainTier::Fast);
    let standard = sample_req(BrainTier::Standard);
    let top = sample_req(BrainTier::Top);
    let (a45, _) = build_cursor_args(&fast, ws, BrainBackend::Grok45);
    let (a45s, _) = build_cursor_args(&standard, ws, BrainBackend::Grok45);
    let (a46, _) = build_cursor_args(&fast, ws, BrainBackend::Grok46);
    let (a46s, _) = build_cursor_args(&standard, ws, BrainBackend::Grok46);
    let (a46t, _) = build_cursor_args(&top, ws, BrainBackend::Grok46);
    assert!(a45.windows(2).any(|w| w == ["--model", "cursor-grok-4.5-high-fast"]));
    assert!(a45s.windows(2).any(|w| w == ["--model", "cursor-grok-4.5-high"]));
    assert!(a46.windows(2).any(|w| w == ["--model", "cursor-grok-4.6-xhigh-fast"]));
    assert!(a46s.windows(2).any(|w| w == ["--model", "cursor-grok-4.6-high"]));
    assert!(a46t.windows(2).any(|w| w == ["--model", "cursor-grok-4.6-xhigh"]));
}

#[test]
fn parse_cursor_json_result() {
    let raw = r#"{"type":"result","subtype":"success","result":"READY","is_error":false}"#;
    assert_eq!(parse_cursor_output(raw).unwrap(), "READY");
}

#[test]
fn parse_cursor_auth_error() {
    let raw = r#"{"type":"result","is_error":true,"result":"Not authenticated. Run agent login."}"#;
    let err = parse_cursor_output(raw).unwrap_err();
    assert!(err.contains("cursor-agent login"));
}

#[test]
fn grok_build_args_prefer_official_cli_flags() {
    let ws = Path::new("/tmp/slate-grok-brain");
    let args = build_grok_build_args(ws, BrainBackend::Grok46);
    assert!(args.windows(2).any(|w| w == ["--output-format", "json"]));
    assert!(args.iter().any(|a| a == "--always-approve"));
    assert!(args.windows(2).any(|w| w == ["--cwd", ws.to_str().unwrap()]));
    assert!(args.windows(2).any(|w| w == ["-m", "grok-4.6"]));
    assert!(args.windows(2).any(|w| w == ["--tools", "read_file"]));
    let args45 = build_grok_build_args(ws, BrainBackend::Grok45);
    assert!(args45.windows(2).any(|w| w == ["-m", "grok-4.5"]));
    assert_eq!(grok_build_cli_model(BrainBackend::Grok46), "grok-4.6");
}

#[test]
fn parse_grok_build_json_text() {
    let raw = r#"{"text":"READY","stopReason":"EndTurn"}"#;
    assert_eq!(parse_grok_build_output(raw).unwrap(), "READY");
}

#[test]
fn parse_grok_build_auth_error() {
    let raw = r#"{"error":"Not authenticated. Run grok login.","is_error":true}"#;
    let err = parse_grok_build_output(raw).unwrap_err();
    assert!(err.contains("grok login"));
}

#[test]
fn grok_auth_json_detects_official_session() {
    let official: serde_json::Value = serde_json::from_str(
        r#"{"https://accounts.x.ai/sign-in":{"key":"session","refresh_token":"r"}}"#,
    )
    .unwrap();
    assert!(grok_auth_looks_signed_in(&official));
    assert!(!grok_auth_looks_signed_in(
        &serde_json::from_str(r#"{"nope":true}"#).unwrap()
    ));
}
