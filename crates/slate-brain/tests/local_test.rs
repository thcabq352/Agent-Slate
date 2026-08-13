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
fn parse_model_ids_openai_and_ollama_tags() {
    let openai = serde_json::json!({"data":[{"id":"qwen3.5:9b"},{"id":"llava:latest"}]});
    assert_eq!(
        slate_brain::local::parse_model_ids(&openai),
        vec!["qwen3.5:9b", "llava:latest"]
    );
    let tags = serde_json::json!({"models":[{"name":"qwen3.5:9b","model":"qwen3.5:9b"}]});
    assert_eq!(
        slate_brain::local::parse_model_ids(&tags),
        vec!["qwen3.5:9b"]
    );
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
