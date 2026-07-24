use crate::ai::{AiProvider, Request};
use crate::commit::{self};
use crate::config::{Config, ConfigChain};
use crate::git::GitBackend;
use crate::search::{self, SearchBackend};
use crate::strutil;
use crate::ui::spinner;
use crate::ui::color;

pub struct WorkflowBuilder {
    git: Option<Box<dyn GitBackend>>,
    ai: Option<Box<dyn AiProvider>>,
    search: Option<Box<dyn SearchBackend>>,
    config: Option<Config>,
    rg_pattern: String,
    fd_pattern: String,
}

impl WorkflowBuilder {
    pub fn new() -> Self {
        Self {
            git: None,
            ai: None,
            search: None,
            config: None,
            rg_pattern: String::new(),
            fd_pattern: String::new(),
        }
    }

    pub fn with_git(mut self, git: impl GitBackend + 'static) -> Self {
        self.git = Some(Box::new(git));
        self
    }

    pub fn with_ai(mut self, ai: impl AiProvider + 'static) -> Self {
        self.ai = Some(Box::new(ai));
        self
    }

    pub fn with_search(mut self, search: impl SearchBackend + 'static) -> Self {
        self.search = Some(Box::new(search));
        self
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_rg_pattern(mut self, pattern: String) -> Self {
        self.rg_pattern = pattern;
        self
    }

    pub fn with_fd_pattern(mut self, pattern: String) -> Self {
        self.fd_pattern = pattern;
        self
    }

    pub fn build(self) -> anyhow::Result<Workflow> {
        Ok(Workflow {
            git: self.git.ok_or_else(|| anyhow::anyhow!("GitBackend 未设置"))?,
            ai: self.ai.ok_or_else(|| anyhow::anyhow!("AiProvider 未设置"))?,
            search: self.search.ok_or_else(|| anyhow::anyhow!("SearchBackend 未设置"))?,
            config: self.config.ok_or_else(|| anyhow::anyhow!("Config 未设置"))?,
            rg_pattern: self.rg_pattern,
            fd_pattern: self.fd_pattern,
        })
    }
}

impl Default for WorkflowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Workflow {
    git: Box<dyn GitBackend>,
    ai: Box<dyn AiProvider>,
    search: Box<dyn SearchBackend>,
    config: Config,
    rg_pattern: String,
    fd_pattern: String,
}

impl Workflow {
    pub fn run(&self) -> anyhow::Result<()> {
        let (type_list, scope_list) = (self.config.types.clone(), self.config.scopes.clone());

        let mut diff = String::new();
        let mut changed_files = Vec::new();
        let mut status_short = String::new();
        let mut diff_stat = String::new();
        let mut diff_cached_stat = String::new();
        let mut untracked_files = Vec::new();

        spinner::run("正在获取 git 变更信息", || {
            diff = self.git.diff()?;
            changed_files = self.git.changed_files().unwrap_or_default();
            status_short = self.git.status_short().unwrap_or_default();
            diff_stat = self.git.diff_stat().unwrap_or_default();
            diff_cached_stat = self.git.diff_cached_stat().unwrap_or_default();
            untracked_files = self.git.ls_untracked().unwrap_or_default();
            Ok(())
        })?;

        if diff.is_empty() && untracked_files.is_empty() {
            println!("没有未暂存的变更");
            return Ok(());
        }

        let mut git_info = String::new();
        if !status_short.is_empty() {
            git_info.push_str(&format!("--- 工作区状态 ---\n{status_short}\n"));
        }
        if !diff_cached_stat.is_empty() {
            git_info.push_str(&format!("--- 已暂存变更摘要 ---\n{diff_cached_stat}\n"));
        }
        if !diff_stat.is_empty() {
            git_info.push_str(&format!("--- 未暂存变更摘要 ---\n{diff_stat}\n"));
        }
        if !untracked_files.is_empty() {
            let content = self.git.untracked_content(&untracked_files);
            if !content.is_empty() {
                git_info.push_str(&format!("--- 未跟踪文件内容 ---\n{content}\n"));
            }
        }

        let rg_context = self.search.rg_context(&changed_files, &self.rg_pattern);
        let fd_context = self.search.fd_context(&changed_files, &self.fd_pattern);
        let extra_context = search::merge_context(&rg_context, &fd_context);

        let mut raw = String::new();
        spinner::run("正在调用 AI 生成 commit 消息", || {
            let r = self.ai.generate(&Request {
                types: type_list.clone(),
                scopes: scope_list.clone(),
                diff: diff.clone(),
                git_info: git_info.clone(),
                extra_context: extra_context.clone(),
            })?;
            raw = r;
            Ok(())
        })?;

        let (entries, reason) = commit::service::parse_entries(&raw);
        if entries.is_empty() {
            anyhow::bail!("AI 未生成有效 commit 消息");
        }
        if !reason.is_empty() {
            println!("\n{}", color::yellow(&reason));
        }

        let total = entries.len();
        for (i, mut entry) in entries.into_iter().enumerate() {
            entry.entry_type = strutil::capitalize(&entry.entry_type);
            entry.scope = strutil::capitalize(&entry.scope);
            let msg = commit::service::format_message(&entry);

            if total > 1 {
                println!("\n{}", color::bold_cyan(&format!("--- 提交 {}/{} ---", i + 1, total)));
            }

            if !commit::service::confirm_entry(&entry) {
                println!("{}", color::yellow("已取消"));
                return Ok(());
            }

            if !entry.files.is_empty() {
                self.git.stage_files(&entry.files)?;
            } else {
                self.git.stage_all()?;
            }
            self.git.commit(&msg)?;
        }

        println!("{}", color::green("全部提交成功"));
        Ok(())
    }
}

pub fn build_default_workflow(
    skill_path: &str,
    lazygit_path: &str,
    api_key: &str,
    rg_pattern: &str,
    fd_pattern: &str,
) -> anyhow::Result<Workflow> {
    let api_key = if !api_key.is_empty() {
        api_key.to_string()
    } else {
        std::env::var("MESSAGE_API_KEY")
            .map_err(|_| anyhow::anyhow!("MESSAGE_API_KEY 未设置，可通过 --api-key 参数或 MESSAGE_API_KEY 环境变量设置"))?
    };
    let model = std::env::var("OPENAI_MODEL")
        .unwrap_or_else(|_| "deepseek-v4-flash".to_string());
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.deepseek.com".to_string());

    let config = ConfigChain::new(lazygit_path, skill_path).load();
    let ai = crate::ai::openai::OpenAI::new(api_key, model, base_url);
    let git = crate::git::real::RealGit;
    let search = crate::search::RealSearch;

    WorkflowBuilder::new()
        .with_git(git)
        .with_ai(ai)
        .with_search(search)
        .with_config(config)
        .with_rg_pattern(rg_pattern.to_string())
        .with_fd_pattern(fd_pattern.to_string())
        .build()
}
