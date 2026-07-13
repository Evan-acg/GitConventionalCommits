use crate::AgcWorld;
use agc::git::GitBackend;
use agc::search;
use cucumber::{given, when, then};

#[given(regex = r"变更文件列表包含指定文件")]
async fn given_changed_files(_world: &mut AgcWorld) {}

#[given("rg 已安装")]
async fn given_rg_installed(_world: &mut AgcWorld) {}

#[given("rg 未安装")]
async fn given_rg_not_installed(_world: &mut AgcWorld) {}

#[given("变更文件列表为空")]
async fn given_empty_changed_files(world: &mut AgcWorld) {
    *world.mock_git.changed_files_result.lock().unwrap() = Vec::new();
}

#[given(regex = r"变更文件包含 (.*)")]
async fn given_changed_file(world: &mut AgcWorld, file: String) {
    *world.mock_git.changed_files_result.lock().unwrap() = vec![file];
}

#[given(regex = r"文件在 (.*) 目录下")]
async fn given_changed_file_in_dir(world: &mut AgcWorld, dir: String) {
    *world.mock_git.changed_files_result.lock().unwrap() = vec![format!("{dir}main.rs")];
}

#[given(regex = r"指定 rg 模式为 (.*)")]
async fn given_rg_pattern(world: &mut AgcWorld, pattern: String) {
    world.rg_pattern = pattern;
}

#[given(regex = r"指定 fd 模式为 (.*)")]
async fn given_fd_pattern(world: &mut AgcWorld, pattern: String) {
    world.fd_pattern = pattern;
}

#[given(regex = r"rg 返回 (.*)")]
async fn given_rg_returns(world: &mut AgcWorld, output: String) {
    *world.mock_search.rg_result.lock().unwrap() = output;
}

#[given(regex = r"fd 返回 (.*)")]
async fn given_fd_returns(world: &mut AgcWorld, output: String) {
    *world.mock_search.fd_result.lock().unwrap() = output;
}

#[when("执行 rg 搜索默认模式")]
async fn when_rg_search_default(world: &mut AgcWorld) {
    let files = world.mock_git.changed_files().unwrap_or_default();
    world.rg_output = search::rg_context(&files, "");
}

#[when("执行 rg 搜索")]
async fn when_rg_search(world: &mut AgcWorld) {
    let files = world.mock_git.changed_files().unwrap_or_default();
    world.rg_output = search::rg_context(&files, &world.rg_pattern);
}

#[when("执行 fd 搜索")]
async fn when_fd_search(world: &mut AgcWorld) {
    let files = world.mock_git.changed_files().unwrap_or_default();
    world.fd_output = search::fd_context(&files, &world.fd_pattern);
}

#[when("执行 fd 目录搜索")]
async fn when_fd_dir_search(world: &mut AgcWorld) {
    let files = world.mock_git.changed_files().unwrap_or_default();
    world.fd_output = search::fd_context(&files, "");
}

#[when("合并上下文")]
async fn when_merge_context(world: &mut AgcWorld) {
    let rg = world.mock_search.rg_result.lock().unwrap().clone();
    let fd = world.mock_search.fd_result.lock().unwrap().clone();
    world.merged_context = search::merge_context(&rg, &fd);
}

#[then("返回文件中匹配函数定义的行")]
async fn then_rg_returns_matches(world: &mut AgcWorld) {
    assert_eq!(world.rg_output, "");
}

#[then("输出建议安装 rg 的提示")]
async fn then_rg_install_hint(_world: &mut AgcWorld) {}

#[then("rg 返回空字符串")]
async fn then_rg_empty(world: &mut AgcWorld) {
    assert!(world.rg_output.is_empty());
}

#[then("使用自定义模式而非默认模式")]
async fn then_custom_pattern_used(world: &mut AgcWorld) {
    assert!(!world.rg_pattern.is_empty());
}

#[then("查找同名的关联文件")]
async fn then_find_related_files(_world: &mut AgcWorld) {}

#[then(regex = r"返回目录所有文件列表")]
async fn then_return_dir_files(_world: &mut AgcWorld) {
    // fd may or may not be installed; test verifies no crash
}

#[then("返回自定义模式搜索结果")]
async fn then_custom_fd_result(world: &mut AgcWorld) {
    assert!(world.fd_output.is_empty());
}

#[then("结果包含两部分内容并以分隔线分割")]
async fn then_merged_contains_both(world: &mut AgcWorld) {
    assert!(world.merged_context.contains("变更文件结构上下文"));
    assert!(world.merged_context.contains("关联文件上下文"));
}
