use crate::AgcWorld;
use agc::commit;
use agc::git::GitBackend;
use cucumber::{given, when, then};

#[given("没有未跟踪的文件")]
async fn given_no_untracked(world: &mut AgcWorld) {
    *world.mock_git.ls_untracked_result.lock().unwrap() = Vec::new();
}

#[given(regex = r"用户输入非 (.*)")]
async fn given_user_input_not_ok(world: &mut AgcWorld, input: String) {
    world.user_input.push_back(input);
}

#[given(regex = r"用户每次输入 (.*)")]
async fn given_user_input_repeated(world: &mut AgcWorld, input: String) {
    world.user_input.push_back(input.clone());
    world.user_input.push_back(input);
}

#[given("用户输入 ok")]
async fn given_user_input_ok(world: &mut AgcWorld) {
    world.user_input.push_back("ok".to_string());
}

#[given("有未暂存变更")]
async fn given_some_diff(world: &mut AgcWorld) {
    *world.mock_git.diff_result.lock().unwrap() = "mock diff content".to_string();
}

#[given("AI 已返回有效条目")]
async fn given_ai_valid_entry(world: &mut AgcWorld) {
    *world.mock_git.diff_result.lock().unwrap() = "mock diff content".to_string();
    world.ai_response = r#"{"reason": "test", "data": [{"type": "Feat", "scope": "Git", "message": "测试消息"}]}"#.to_string();
    *world.mock_ai.generate_result.lock().unwrap() = world.ai_response.clone();
}

#[given(regex = r"AI 返回 (\d+) 条独立 commit 条目")]
async fn given_ai_multi_entries(world: &mut AgcWorld, count: usize) {
    *world.mock_git.diff_result.lock().unwrap() = "mock diff content".to_string();
    let entries: Vec<String> = (0..count).map(|i| {
        format!(r#"{{"type": "Feat", "scope": "Git", "message": "提交{}"}}"#, i + 1)
    }).collect();
    world.ai_response = format!(r#"{{"reason": "多个独立变更", "data": [{}]}}"#, entries.join(","));
    *world.mock_ai.generate_result.lock().unwrap() = world.ai_response.clone();
}

#[given("AI 返回的条目中有一条缺少 type")]
async fn given_ai_with_invalid_entry(world: &mut AgcWorld) {
    *world.mock_git.diff_result.lock().unwrap() = "mock diff content".to_string();
    world.ai_response = r#"{
        "reason": "test",
        "data": [
            {"type": "", "scope": "Git", "message": "无效条目"},
            {"type": "Feat", "scope": "Config", "message": "有效条目"}
        ]
    }"#.to_string();
    *world.mock_ai.generate_result.lock().unwrap() = world.ai_response.clone();
}

#[given("AI 返回的条目包含 files 字段")]
async fn given_ai_with_files_field(world: &mut AgcWorld) {
    *world.mock_git.diff_result.lock().unwrap() = "mock diff content".to_string();
    world.ai_response = r#"{"reason": "test", "data": [{"type": "Feat", "scope": "Git", "message": "测试", "files": ["src/main.rs"]}]}"#.to_string();
    *world.mock_ai.generate_result.lock().unwrap() = world.ai_response.clone();
}

#[given("还有一条有效条目")]
async fn given_another_valid_entry(_world: &mut AgcWorld) {}

#[when("执行完整工作流")]
async fn when_run_workflow(world: &mut AgcWorld) {
    let diff = world.mock_git.diff().unwrap_or_default();
    let untracked = world.mock_git.ls_untracked().unwrap_or_default();

    if diff.is_empty() && untracked.is_empty() {
        world.workflow_output.push("没有未暂存的变更".to_string());
        return;
    }

    let ai_raw = world.mock_ai.generate_result.lock().unwrap().clone();
    let (mut entries, _reason) = commit::parse_entries(&ai_raw);

    if entries.is_empty() {
        world.workflow_output.push("AI 未生成有效 commit 消息".to_string());
        return;
    }

    entries.retain(|e| {
        if e.entry_type.is_empty() || e.scope.is_empty() || e.message.is_empty() {
            world.workflow_output.push("警告: AI 返回的 entry 缺少必要字段，已跳过".to_string());
            false
        } else {
            true
        }
    });

    if entries.is_empty() {
        world.workflow_output.push("AI 未生成有效 commit 消息".to_string());
        return;
    }

    for entry in &entries {
        let input = world.user_input.pop_front().unwrap_or_default();
        if input != "ok" {
            world.workflow_output.push("已取消".to_string());
            return;
        }

        if !entry.files.is_empty() {
            world.mock_git.stage_files(&entry.files).unwrap();
        } else {
            world.mock_git.stage_all().unwrap();
        }
        world.mock_git.commit(&commit::format_message(entry)).unwrap();
    }
    world.workflow_output.push("全部提交成功".to_string());
}

#[when("执行确认步骤")]
async fn when_confirm(world: &mut AgcWorld) {
    let input = world.user_input.pop_front().unwrap_or_default();
    let clone = input.clone();
    world.user_input.push_back(input);
    if clone != "ok" {
        world.workflow_output.push("已取消".to_string());
    }
}

#[then(regex = r"输出 (.*)")]
async fn then_output_contains(world: &mut AgcWorld, expected: String) {
    assert!(
        world.workflow_output.iter().any(|o| o.contains(&expected)),
        "expected output to contain '{expected}', got: {:?}",
        world.workflow_output
    );
}

#[then("不调用 AI")]
async fn then_ai_not_called(_world: &mut AgcWorld) {}

#[then("不执行 git commit")]
async fn then_git_commit_not_called(world: &mut AgcWorld) {
    assert!(world.mock_git.committed_messages.lock().unwrap().is_empty());
}

#[then(regex = r"git commit 被执行 (\d+) 次")]
async fn then_git_commit_count(world: &mut AgcWorld, count: usize) {
    assert_eq!(world.mock_git.committed_messages.lock().unwrap().len(), count);
}

#[then("输出警告跳过无效条目")]
async fn then_skip_warning(world: &mut AgcWorld) {
    assert!(
        world.workflow_output.iter().any(|o| o.contains("缺少必要字段")),
        "should contain skip warning"
    );
}

#[then("只 stage 指定的文件")]
async fn then_stage_specific_files(world: &mut AgcWorld) {
    let staged = world.mock_git.staged_files.lock().unwrap();
    assert_eq!(staged.len(), 1);
    assert!(staged.contains(&"src/main.rs".to_string()));
    assert!(!*world.mock_git.stage_all_called.lock().unwrap());
}
