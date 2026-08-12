use slate_brain::{build_claude_args, parse_claude_output, BrainRequest, BrainTier};

#[test]
fn claude_args_include_print_and_json() {
    let req = BrainRequest {
        id: "1".into(),
        task: "t".into(),
        system: "sys".into(),
        prompt: "hello".into(),
        images: vec![],
        tier: BrainTier::Fast,
        expect_json: false,
        local_endpoint: None,
        local_model: None,
    };
    let (args, input) = build_claude_args(&req);
    assert!(args.iter().any(|a| a == "-p"));
    assert!(args.windows(2).any(|w| w == ["--output-format", "json"]));
    assert!(args.windows(2).any(|w| w == ["--model", "haiku"]));
    assert_eq!(input, "hello");
}

#[test]
fn parse_claude_json_result() {
    let raw = r#"{"type":"result","result":"READY","is_error":false}"#;
    assert_eq!(parse_claude_output(raw).unwrap(), "READY");
}
