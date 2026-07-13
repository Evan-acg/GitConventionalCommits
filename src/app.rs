use crate::ai::{AiProvider, Request};
use crate::commit;
use crate::config;
use crate::git::{GitBackend, RealGit};
use crate::search;
use crate::spinner;
use crate::strutil;

pub struct Options {
    pub skill_path: String,
    pub api_key: String,
    pub rg_pattern: String,
    pub fd_pattern: String,
    pub lazygit_path: String,
}

pub struct WorkflowContext;

pub struct Workflow;

impl Workflow {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, opts: Options) -> anyhow::Result<()> {
        let (type_list, scope_list) = config::load(&opts.skill_path, &opts.lazygit_path);

        let git = RealGit;

        let mut diff = String::new();
        let mut changed_files = Vec::new();
        let mut status_short = String::new();
        let mut diff_stat = String::new();
        let mut diff_cached_stat = String::new();
        let mut untracked_files = Vec::new();

        spinner::run("正在获取 git 变更信息", || {
            diff = git.diff()?;
            changed_files = git.changed_files().unwrap_or_default();
            status_short = git.status_short().unwrap_or_default();
            diff_stat = git.diff_stat().unwrap_or_default();
            diff_cached_stat = git.diff_cached_stat().unwrap_or_default();
            untracked_files = git.ls_untracked().unwrap_or_default();
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
            let content = git.untracked_content(&untracked_files);
            if !content.is_empty() {
                git_info.push_str(&format!("--- 未跟踪文件内容 ---\n{content}\n"));
            }
        }

        let rg_context = search::rg_context(&changed_files, &opts.rg_pattern);
        let fd_context = search::fd_context(&changed_files, &opts.fd_pattern);
        let extra_context = search::merge_context(&rg_context, &fd_context);

        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "deepseek-v4-flash".to_string());

        let api_key = if !opts.api_key.is_empty() {
            opts.api_key.clone()
        } else {
            std::env::var("MESSAGE_API_KEY")
                .map_err(|_| anyhow::anyhow!("MESSAGE_API_KEY 未设置，可通过 --api-key 参数或 MESSAGE_API_KEY 环境变量设置"))?
        };

        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.deepseek.com".to_string());

        let llm = crate::openai::OpenAI::new(api_key, model, base_url);

        let mut raw = String::new();
        spinner::run("正在调用 AI 生成 commit 消息", || {
            let r = futures::executor::block_on(llm.generate(&WorkflowContext, &Request {
                types: type_list.clone(),
                scopes: scope_list.clone(),
                diff: diff.clone(),
                git_info: git_info.clone(),
                extra_context: extra_context.clone(),
            }))?;
            raw = r;
            Ok(())
        })?;

        let (entries, reason) = commit::parse_entries(&raw);
        if entries.is_empty() {
            anyhow::bail!("AI 未生成有效 commit 消息");
        }
        if !reason.is_empty() {
            println!("\n{reason}");
        }

        let entries: Vec<_> = entries
            .into_iter()
            .filter(|e| {
                if e.entry_type.is_empty() || e.scope.is_empty() || e.message.is_empty() {
                    eprintln!("警告: AI 返回的 entry 缺少必要字段，已跳过");
                    false
                } else {
                    true
                }
            })
            .collect();

        if entries.is_empty() {
            anyhow::bail!("AI 未生成有效 commit 消息");
        }

        let total = entries.len();
        for (i, mut entry) in entries.into_iter().enumerate() {
            entry.entry_type = strutil::capitalize(&entry.entry_type);
            entry.scope = strutil::capitalize(&entry.scope);
            let msg = commit::format_message(&entry);

            if total > 1 {
                println!("\n--- 提交 {}/{} ---", i + 1, total);
            }

            if !commit::confirm_entry(&entry) {
                println!("已取消");
                return Ok(());
            }

            if !entry.files.is_empty() {
                git.stage_files(&entry.files)?;
            } else {
                git.stage_all()?;
            }
            git.commit(&msg)?;
        }

        println!("全部提交成功");
        Ok(())
    }
}
