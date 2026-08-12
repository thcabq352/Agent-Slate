use slate_brain::extract_json;

#[test]
fn extracts_object_from_fenced_noise() {
    let t = "Sure!\n```json\n{\"a\":1,\"b\":[2]}\n```\n";
    let v = extract_json(t).unwrap();
    assert_eq!(v["a"], 1);
}

#[test]
fn errors_when_no_json() {
    assert!(extract_json("hello only").is_err());
}
