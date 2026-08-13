mod common;

use agc::commit::Entry;
use agc::pipeline::stages::{CommitExecutor, GitPusher};
use agc::pipeline::{PipelineContext, PipelineStage};
use common::MockGitBackend;
use std::sync::Arc;

fn entry(type_: &str, message: &str) -> Entry {
    Entry {
        entry_type: type_.to_string(),
        scope: "Git".to_string(),
        message: message.to_string(),
        files: vec![],
        detail: String::new(),
    }
}

#[test]
fn auto_commit_commits_without_user_confirmation() {
    let git = Arc::new(MockGitBackend::new());
    let executor = CommitExecutor::new(Arc::clone(&git) as Arc<dyn agc::git::GitBackend>);

    let mut ctx = PipelineContext::new(None, true);
    ctx.has_changes = true;
    ctx.entries = vec![entry("Feat", "添加功能")];

    executor.execute(&mut ctx).unwrap();

    let committed = git.committed_messages.lock().unwrap();
    assert_eq!(committed.len(), 1);
    assert_eq!(committed[0], "Feat(Git): 添加功能");
}

#[test]
fn auto_commit_commits_all_entries_in_loop() {
    let git = Arc::new(MockGitBackend::new());
    let executor = CommitExecutor::new(Arc::clone(&git) as Arc<dyn agc::git::GitBackend>);

    let mut ctx = PipelineContext::new(None, true);
    ctx.has_changes = true;
    ctx.entries = vec![
        entry("Feat", "功能一"),
        entry("Fix", "修复二"),
    ];

    executor.execute(&mut ctx).unwrap();

    assert_eq!(git.committed_messages.lock().unwrap().len(), 2);
}

#[test]
fn push_pushes_remote_and_branch() {
    let git = Arc::new(MockGitBackend::new());
    let pusher = GitPusher::new(
        Arc::clone(&git) as Arc<dyn agc::git::GitBackend>,
        "origin".to_string(),
        "master".to_string(),
    );

    let mut ctx = PipelineContext::new(Some(("origin".to_string(), "master".to_string())), true);
    pusher.execute(&mut ctx).unwrap();

    let calls = git.pushed_calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], ("origin".to_string(), "master".to_string()));
}
