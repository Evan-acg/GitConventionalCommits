use crate::AgcWorld;
use cucumber::{given, when, then};
use agc::git::GitBackend;

#[given("工作区有未暂存的变更")]
async fn given_unstaged_changes(world: &mut AgcWorld) {
    *world.mock_git.diff_result.lock().unwrap() = "diff --git a/src/main.rs b/src/main.rs\nindex abc..def 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n-    println!(\"old\");\n+    println!(\"new\");\n }".to_string();
    *world.mock_git.changed_files_result.lock().unwrap() = vec!["src/main.rs".to_string()];
    *world.mock_git.status_short_result.lock().unwrap() = " M src/main.rs\n".to_string();
    *world.mock_git.diff_stat_result.lock().unwrap() = "1 file changed, 1 insertion(+), 1 deletion(-)\n".to_string();
}

#[given("工作区没有未暂存的变更")]
async fn given_no_changes(world: &mut AgcWorld) {
    *world.mock_git.diff_result.lock().unwrap() = String::new();
    *world.mock_git.status_short_result.lock().unwrap() = String::new();
}

#[given(regex = r"工作区有 (staged|unstaged) 变更")]
async fn given_some_changes(world: &mut AgcWorld) {
    *world.mock_git.diff_result.lock().unwrap() = "diff --git a/src/main.rs b/src/main.rs\n".to_string();
    *world.mock_git.diff_cached_stat_result.lock().unwrap() = "1 file changed\n".to_string();
    *world.mock_git.diff_stat_result.lock().unwrap() = "1 file changed\n".to_string();
    *world.mock_git.status_short_result.lock().unwrap() = "M  src/main.rs\n".to_string();
}

#[given("工作区有文件修改")]
async fn given_file_changes(world: &mut AgcWorld) {
    *world.mock_git.changed_files_result.lock().unwrap() = vec!["src/main.rs".to_string(), "src/lib.rs".to_string()];
}

#[given("工作区有未跟踪的文件")]
async fn given_untracked_files(world: &mut AgcWorld) {
    *world.mock_git.ls_untracked_result.lock().unwrap() = vec!["new_file.rs".to_string()];
}

#[given("存在未跟踪的文本文件小于 100KB")]
async fn given_small_untracked(world: &mut AgcWorld) {
    *world.mock_git.ls_untracked_result.lock().unwrap() = vec!["small_file.txt".to_string()];
    *world.mock_git.untracked_content_result.lock().unwrap() = "--- 未跟踪文件: small_file.txt ---\nhello world\n".to_string();
}

#[given("存在未跟踪的文件超过 100KB")]
async fn given_large_untracked(world: &mut AgcWorld) {
    *world.mock_git.ls_untracked_result.lock().unwrap() = vec!["large_file.bin".to_string()];
    *world.mock_git.untracked_content_result.lock().unwrap() = "--- 未跟踪文件 (超过100KB，已跳过): large_file.bin ---\n".to_string();
}

#[given("存在未跟踪的二进制文件")]
async fn given_binary_untracked(world: &mut AgcWorld) {
    *world.mock_git.ls_untracked_result.lock().unwrap() = vec!["binary_file.exe".to_string()];
    *world.mock_git.untracked_content_result.lock().unwrap() = "--- 未跟踪文件 (二进制): binary_file.exe ---\n".to_string();
}

#[given("不在 git 仓库中")]
async fn given_not_in_repo(world: &mut AgcWorld) {
    *world.mock_git.diff_error.lock().unwrap() = true;
}

#[when("执行 git diff")]
async fn when_git_diff(world: &mut AgcWorld) {
    match world.mock_git.diff() {
        Ok(d) => world.git_info = d,
        Err(e) => world.git_info = format!("ERROR: {e}"),
    }
}

#[when("获取变更文件列表")]
async fn when_changed_files(world: &mut AgcWorld) {
    world.git_files = world.mock_git.changed_files().unwrap_or_default();
}

#[when("获取 status short")]
async fn when_status_short(world: &mut AgcWorld) {
    world.git_info = world.mock_git.status_short().unwrap_or_default();
}

#[when("获取 diff stat")]
async fn when_diff_stat(world: &mut AgcWorld) {
    world.git_info = world.mock_git.diff_stat().unwrap_or_default();
}

#[when("获取 diff cached stat")]
async fn when_diff_cached_stat(world: &mut AgcWorld) {
    world.git_info = world.mock_git.diff_cached_stat().unwrap_or_default();
}

#[when("获取未跟踪文件列表")]
async fn when_ls_untracked(world: &mut AgcWorld) {
    world.untracked_files = world.mock_git.ls_untracked().unwrap_or_default();
}

#[when("读取未跟踪文件内容")]
async fn when_untracked_content(world: &mut AgcWorld) {
    let paths = world.mock_git.ls_untracked().unwrap_or_default();
    world.git_info = world.mock_git.untracked_content(&paths);
}

#[then("返回包含变更内容的 diff 字符串")]
async fn then_diff_not_empty(world: &mut AgcWorld) {
    assert!(world.git_info.contains("diff --git"), "should contain git diff header");
}

#[then("返回空字符串的 git 信息")]
async fn then_empty_git_info(world: &mut AgcWorld) {
    assert!(world.git_info.is_empty(), "should be empty, got: {:?}", world.git_info);
}

#[then("返回只包含已修改文件路径的列表")]
async fn then_file_list(world: &mut AgcWorld) {
    assert!(world.git_files.contains(&"src/main.rs".to_string()));
}

#[then("返回 git status short 格式的输出")]
async fn then_status_short(world: &mut AgcWorld) {
    assert!(!world.git_info.is_empty());
}

#[then("返回包含插入删除行数的统计信息")]
async fn then_diff_stat(world: &mut AgcWorld) {
    assert!(!world.git_info.is_empty());
}

#[then("返回已暂存变更的统计信息")]
async fn then_cached_stat(world: &mut AgcWorld) {
    assert!(!world.git_info.is_empty());
}

#[then("返回未跟踪文件的路径列表")]
async fn then_untracked_list(world: &mut AgcWorld) {
    assert!(world.untracked_files.contains(&"new_file.rs".to_string()));
}

#[then("返回包含文件内容的字符串")]
async fn then_content_string(world: &mut AgcWorld) {
    assert!(world.git_info.contains("hello world"));
}

#[then("跳过该文件并给出跳过提示")]
async fn then_skip_large(world: &mut AgcWorld) {
    assert!(world.git_info.contains("超过100KB"));
}

#[then("跳过该文件并给出二进制提示")]
async fn then_skip_binary(world: &mut AgcWorld) {
    assert!(world.git_info.contains("二进制"));
}

#[then("返回错误的 diff")]
async fn then_error_diff(world: &mut AgcWorld) {
    assert!(world.git_info.contains("ERROR"), "should contain error");
}
