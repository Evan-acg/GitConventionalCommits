use crate::AgcWorld;
use cucumber::{given, when, then};
use agc::commit;

#[given("AI 返回单条 JSON 响应")]
async fn given_ai_single_response(world: &mut AgcWorld) {
    world.ai_response = r#"{"reason": "测试原因", "data": [{"type": "Feat", "scope": "Git", "message": "测试消息"}]}"#.to_string();
}

#[given("AI 返回包含 2 条 data 的 JSON")]
async fn given_ai_multi_response(world: &mut AgcWorld) {
    world.ai_response = r#"{
        "reason": "包含两个独立变更",
        "data": [
            {"type": "Feat", "scope": "Git", "message": "第一个提交"},
            {"type": "Fix", "scope": "Config", "message": "第二个提交"}
        ]
    }"#.to_string();
}

#[given("AI 返回带 reason 的 JSON")]
async fn given_ai_with_reason(world: &mut AgcWorld) {
    world.ai_response = r#"{"reason": "分析说明", "data": [{"type": "Feat", "scope": "Git", "message": "消息"}]}"#.to_string();
}

#[given("AI 返回带 files 的 JSON")]
async fn given_ai_with_files(world: &mut AgcWorld) {
    world.ai_response = r#"{
        "reason": "test",
        "data": [{"type": "Feat", "scope": "Git", "message": "消息", "files": ["src/main.rs", "src/lib.rs"]}]
    }"#.to_string();
}

#[given("AI 返回带 detail 的 JSON")]
async fn given_ai_with_detail(world: &mut AgcWorld) {
    world.ai_response = r#"{
        "reason": "test",
        "data": [{"type": "Feat", "scope": "Git", "message": "消息", "detail": "- 第一项\n- 第二项"}]
    }"#.to_string();
}

#[given("AI 返回非 JSON 字符串")]
async fn given_ai_invalid_response(world: &mut AgcWorld) {
    world.ai_response = "这不是 JSON".to_string();
}

#[given("AI 返回空 JSON 对象")]
async fn given_ai_empty_response(world: &mut AgcWorld) {
    world.ai_response = "{}".to_string();
}

#[when("解析响应")]
async fn when_parse_response(world: &mut AgcWorld) {
    let (entries, reason) = commit::parse_entries(&world.ai_response);
    world.entries = entries;
    world.reason = reason;
}

#[then(regex = r"返回 (\d+) 条 Entry")]
async fn then_entry_count(world: &mut AgcWorld, count: usize) {
    assert_eq!(world.entries.len(), count, "entry count mismatch");
}

#[then("Entry type 为 Feat")]
async fn then_entry_type(world: &mut AgcWorld) {
    assert_eq!(world.entries[0].entry_type, "Feat");
}

#[then("Entry scope 为 Git")]
async fn then_entry_scope(world: &mut AgcWorld) {
    assert_eq!(world.entries[0].scope, "Git");
}

#[then("同时返回 reason 字符串")]
async fn then_reason_returned(world: &mut AgcWorld) {
    assert_eq!(world.reason, "分析说明");
}

#[then("Entry files 包含指定的文件路径")]
async fn then_files_contained(world: &mut AgcWorld) {
    assert!(world.entries[0].files.contains(&"src/main.rs".to_string()));
}

#[then("Entry detail 包含详细描述")]
async fn then_detail_contained(world: &mut AgcWorld) {
    assert!(world.entries[0].detail.contains("第一项"));
}

#[then("返回空 entries 列表")]
async fn then_empty_entries(world: &mut AgcWorld) {
    assert!(world.entries.is_empty(), "entries should be empty");
}
