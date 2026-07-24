use crate::AgcWorld;
use cucumber::{given, when, then};
use agc::commit::Entry;
use agc::strutil;

#[given("单个 Entry 包含 type scope message")]
async fn given_single_entry(world: &mut AgcWorld) {
    world.current_entry = Some(Entry {
        entry_type: "Feat".to_string(),
        scope: "Git".to_string(),
        message: "添加 diff 函数".to_string(),
        files: vec![],
        detail: String::new(),
    });
}

#[given("Entry 带详细描述")]
async fn given_entry_with_detail(world: &mut AgcWorld) {
    world.current_entry = Some(Entry {
        entry_type: "Fix".to_string(),
        scope: "Config".to_string(),
        message: "修复配置加载".to_string(),
        files: vec![],
        detail: "- 修复 YAML 解析\n- 添加错误处理".to_string(),
    });
}

#[given(regex = r"原始字符串 (.*)")]
async fn given_raw_string(world: &mut AgcWorld, s: String) {
    world.current_raw = s;
}

#[given("输入的字符串为空")]
async fn given_empty_string(world: &mut AgcWorld) {
    world.current_raw = String::new();
}

#[given("存在 entries 其中一个缺少 message 字段")]
async fn given_entries_with_missing_field(world: &mut AgcWorld) {
    world.entries = vec![
        Entry {
            entry_type: "Feat".to_string(),
            scope: "Git".to_string(),
            message: "valid".to_string(),
            files: vec![],
            detail: String::new(),
        },
        Entry {
            entry_type: "Fix".to_string(),
            scope: "Config".to_string(),
            message: String::new(),
            files: vec![],
            detail: String::new(),
        },
    ];
}

#[when("格式化消息")]
async fn when_format(world: &mut AgcWorld) {
    if let Some(ref entry) = world.current_entry {
        world.formatted_msg = agc::commit::service::format_message(entry);
    }
}

#[when("应用 Capitalize 处理")]
async fn when_capitalize(world: &mut AgcWorld) {
    world.current_raw = strutil::capitalize(&world.current_raw);
}

#[when("验证条目有效性")]
async fn when_validate_entries(world: &mut AgcWorld) {
    let valid: Vec<_> = world.entries.iter()
        .filter(|e| !e.entry_type.is_empty() && !e.scope.is_empty() && !e.message.is_empty())
        .cloned()
        .collect();
    if world.entries.len() != valid.len() {
        world.workflow_output.push("警告: AI 返回的 entry 缺少必要字段，已跳过".to_string());
    }
    world.entries = valid;
}

#[then(regex = r"格式化结果为 (.*)")]
async fn then_format_result(world: &mut AgcWorld, expected: String) {
    assert_eq!(world.formatted_msg, expected, "formatted message mismatch");
}

#[then(regex = r"字符串结果为 (.*)")]
async fn then_string_result(world: &mut AgcWorld, expected: String) {
    assert_eq!(world.current_raw, expected, "string result mismatch");
}

#[then("字符串结果为空")]
async fn then_empty_result(world: &mut AgcWorld) {
    assert!(world.current_raw.is_empty(), "should be empty");
}

#[then("detail 包含在格式化结果中")]
async fn then_detail_included(world: &mut AgcWorld) {
    assert!(world.formatted_msg.contains("修复 YAML 解析"), "detail should be included");
    assert!(world.formatted_msg.contains("添加错误处理"), "detail should be included");
}

#[then("该条目被过滤并输出警告")]
async fn then_filtered_with_warning(world: &mut AgcWorld) {
    assert_eq!(world.entries.len(), 1, "should have 1 valid entry after filter");
    assert!(
        world.workflow_output.iter().any(|o| o.contains("缺少必要字段")),
        "should output warning"
    );
}
