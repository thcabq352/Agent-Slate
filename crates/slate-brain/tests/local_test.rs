//! Local OpenAI-compatible adapter tests (no network).

#[test]
fn parse_chat_response_reads_content() {
    let body = r#"{"choices":[{"message":{"content":"  hi  "}}]}"#;
    assert_eq!(
        slate_brain::local::parse_chat_response(body).unwrap(),
        "hi"
    );
}

#[test]
fn parse_chat_response_errors_on_empty() {
    let body = r#"{"choices":[{"message":{"content":"   "}}]}"#;
    assert!(slate_brain::local::parse_chat_response(body).is_err());
}

#[test]
fn parse_chat_response_surfaces_api_error() {
    let body = r#"{"error":{"message":"model not found"}}"#;
    let err = slate_brain::local::parse_chat_response(body).unwrap_err();
    assert!(err.contains("model not found"));
}

#[test]
fn normalize_endpoint_adds_scheme_and_v1() {
    assert_eq!(
        slate_brain::local::normalize_endpoint("localhost:1234"),
        "http://localhost:1234/v1"
    );
    assert_eq!(
        slate_brain::local::normalize_endpoint("http://localhost:11434/v1/"),
        "http://localhost:11434/v1"
    );
}
