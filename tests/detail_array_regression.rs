use agc::commit::service;

// Given AI returns an entry whose detail field is an array of strings
// When the response is parsed
// Then the entry detail is normalized to a Markdown list string
#[test]
fn normalizes_detail_array_to_markdown_list() {
    let response = r#"{
        "reason": "测试",
        "data": [{
            "type": "Fix",
            "scope": "Ai",
            "message": "兼容 detail 数组",
            "detail": ["修改一", "修改二"]
        }]
    }"#;

    let (entries, _) = service::parse_entries(response);

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].detail, "- 修改一\n- 修改二");
}

#[test]
fn preserves_detail_string() {
    let response = r#"{
        "reason": "测试",
        "data": [{
            "type": "Fix",
            "scope": "Ai",
            "message": "保留 detail 字符串",
            "detail": "- 修改一\n- 修改二"
        }]
    }"#;

    let (entries, _) = service::parse_entries(response);

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].detail, "- 修改一\n- 修改二");
}

#[test]
fn rejects_non_string_detail_values() {
    let response = r#"{
        "reason": "测试",
        "data": [{
            "type": "Fix",
            "scope": "Ai",
            "message": "拒绝无效 detail",
            "detail": {"text": "修改一"}
        }]
    }"#;

    let (entries, _) = service::parse_entries(response);

    assert!(entries.is_empty());
}
