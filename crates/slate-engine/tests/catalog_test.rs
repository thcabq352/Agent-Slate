#[test]
fn tool_catalog_includes_health() {
    let names: Vec<_> = slate_engine::tools::catalog()
        .into_iter()
        .map(|t| t.name)
        .collect();
    assert!(names.iter().any(|n| n == "slate_health"));
    assert!(names.iter().any(|n| n == "slate_judge_take"));
}

#[test]
fn tool_catalog_entries_have_input_schema() {
    for tool in slate_engine::tools::catalog() {
        assert_eq!(
            tool.input_schema.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "tool {} inputSchema.type",
            tool.name
        );
    }
}
