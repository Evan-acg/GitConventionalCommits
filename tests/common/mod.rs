use agc::ai::{AiProvider, Request};
use agc::commit::Entry;
use agc::git::GitBackend;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct MockGitBackend {
    pub diff_result: Arc<Mutex<String>>,
    pub diff_error: Arc<Mutex<bool>>,
    pub changed_files_result: Arc<Mutex<Vec<String>>>,
    pub status_short_result: Arc<Mutex<String>>,
    pub diff_stat_result: Arc<Mutex<String>>,
    pub diff_cached_stat_result: Arc<Mutex<String>>,
    pub ls_untracked_result: Arc<Mutex<Vec<String>>>,
    pub untracked_content_result: Arc<Mutex<String>>,
    pub staged_files: Arc<Mutex<Vec<String>>>,
    pub committed_messages: Arc<Mutex<Vec<String>>>,
    pub stage_all_called: Arc<Mutex<bool>>,
}

impl MockGitBackend {
    pub fn new() -> Self {
        Self {
            diff_result: Arc::new(Mutex::new(String::new())),
            diff_error: Arc::new(Mutex::new(false)),
            changed_files_result: Arc::new(Mutex::new(Vec::new())),
            status_short_result: Arc::new(Mutex::new(String::new())),
            diff_stat_result: Arc::new(Mutex::new(String::new())),
            diff_cached_stat_result: Arc::new(Mutex::new(String::new())),
            ls_untracked_result: Arc::new(Mutex::new(Vec::new())),
            untracked_content_result: Arc::new(Mutex::new(String::new())),
            staged_files: Arc::new(Mutex::new(Vec::new())),
            committed_messages: Arc::new(Mutex::new(Vec::new())),
            stage_all_called: Arc::new(Mutex::new(false)),
        }
    }
}

impl GitBackend for MockGitBackend {
    fn diff(&self) -> anyhow::Result<String> {
        let err = *self.diff_error.lock().unwrap();
        if err {
            anyhow::bail!("mock git diff error")
        }
        Ok(self.diff_result.lock().unwrap().clone())
    }
    fn changed_files(&self) -> anyhow::Result<Vec<String>> {
        Ok(self.changed_files_result.lock().unwrap().clone())
    }
    fn status_short(&self) -> anyhow::Result<String> {
        Ok(self.status_short_result.lock().unwrap().clone())
    }
    fn diff_stat(&self) -> anyhow::Result<String> {
        Ok(self.diff_stat_result.lock().unwrap().clone())
    }
    fn diff_cached_stat(&self) -> anyhow::Result<String> {
        Ok(self.diff_cached_stat_result.lock().unwrap().clone())
    }
    fn ls_untracked(&self) -> anyhow::Result<Vec<String>> {
        Ok(self.ls_untracked_result.lock().unwrap().clone())
    }
    fn untracked_content(&self, _paths: &[String]) -> String {
        self.untracked_content_result.lock().unwrap().clone()
    }
    fn stage_all(&self) -> anyhow::Result<()> {
        *self.stage_all_called.lock().unwrap() = true;
        Ok(())
    }
    fn stage_files(&self, files: &[String]) -> anyhow::Result<()> {
        let mut sf = self.staged_files.lock().unwrap();
        sf.extend_from_slice(files);
        Ok(())
    }
    fn commit(&self, msg: &str) -> anyhow::Result<()> {
        let mut cm = self.committed_messages.lock().unwrap();
        cm.push(msg.to_string());
        Ok(())
    }
    fn push(&self, _remote: &str) -> anyhow::Result<()> {
        Ok(())
    }
    fn pull(&self, _remote: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct MockAiProvider {
    pub generate_result: Arc<Mutex<String>>,
    pub generate_error: Arc<Mutex<bool>>,
}

impl MockAiProvider {
    pub fn new() -> Self {
        Self {
            generate_result: Arc::new(Mutex::new(String::new())),
            generate_error: Arc::new(Mutex::new(false)),
        }
    }
}

impl AiProvider for MockAiProvider {
    fn generate(&self, _req: &Request) -> anyhow::Result<String> {
        let err = *self.generate_error.lock().unwrap();
        if err {
            anyhow::bail!("mock ai error")
        }
        Ok(self.generate_result.lock().unwrap().clone())
    }
}

#[derive(Debug)]
pub struct MockSearchBackend {
    pub rg_result: Arc<Mutex<String>>,
    pub fd_result: Arc<Mutex<String>>,
}

impl MockSearchBackend {
    pub fn new() -> Self {
        Self {
            rg_result: Arc::new(Mutex::new(String::new())),
            fd_result: Arc::new(Mutex::new(String::new())),
        }
    }
}

#[derive(Debug, cucumber::World)]
pub struct AgcWorld {
    // Config
    pub temp_dir: Option<tempfile::TempDir>,
    pub loaded_types: Vec<String>,
    pub loaded_scopes: Vec<String>,
    pub config_skill_path: String,
    pub config_lazygit_path: String,

    // Init
    pub init_dir: Option<tempfile::TempDir>,
    pub init_path: String,
    pub init_error: Option<String>,

    // Git
    pub mock_git: MockGitBackend,
    pub git_info: String,
    pub git_files: Vec<String>,
    pub untracked_files: Vec<String>,

    // AI
    pub mock_ai: MockAiProvider,
    pub ai_response: String,

    // Search
    pub mock_search: MockSearchBackend,
    pub rg_output: String,
    pub fd_output: String,
    pub merged_context: String,
    pub rg_pattern: String,
    pub fd_pattern: String,

    // Commit
    pub entries: Vec<Entry>,
    pub reason: String,
    pub formatted_msg: String,
    pub current_entry: Option<Entry>,
    pub current_raw: String,

    // Workflow
    pub user_input: VecDeque<String>,
    pub workflow_output: Vec<String>,
    pub commit_count: usize,
    pub auto_commit: bool,
}

impl Default for AgcWorld {
    fn default() -> Self {
        Self {
            temp_dir: None,
            loaded_types: Vec::new(),
            loaded_scopes: Vec::new(),
            config_skill_path: String::new(),
            config_lazygit_path: String::new(),
            init_dir: None,
            init_path: String::new(),
            init_error: None,
            mock_git: MockGitBackend::new(),
            git_info: String::new(),
            git_files: Vec::new(),
            untracked_files: Vec::new(),
            mock_ai: MockAiProvider::new(),
            ai_response: String::new(),
            mock_search: MockSearchBackend::new(),
            rg_output: String::new(),
            fd_output: String::new(),
            merged_context: String::new(),
            rg_pattern: String::new(),
            fd_pattern: String::new(),
            entries: Vec::new(),
            reason: String::new(),
            formatted_msg: String::new(),
            current_entry: None,
            current_raw: String::new(),
            user_input: VecDeque::new(),
            workflow_output: Vec::new(),
            commit_count: 0,
            auto_commit: false,
        }
    }
}
