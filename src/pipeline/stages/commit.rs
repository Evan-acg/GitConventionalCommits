use std::sync::Arc;

use crate::commit::{self};
use crate::git::GitBackend;
use crate::pipeline::{PipelineContext, PipelineStage};
use crate::strutil;
use crate::ui::color;

pub struct CommitExecutor {
    git: Arc<dyn GitBackend>,
}

impl CommitExecutor {
    pub fn new(git: Arc<dyn GitBackend>) -> Self {
        Self { git }
    }
}

impl PipelineStage for CommitExecutor {
    fn execute(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        if !ctx.has_changes || ctx.entries.is_empty() {
            return Ok(());
        }

        let total = ctx.entries.len();
        let entries = std::mem::take(&mut ctx.entries);

        for (i, mut entry) in entries.into_iter().enumerate() {
            entry.entry_type = strutil::capitalize(&entry.entry_type);
            entry.scope = strutil::capitalize(&entry.scope);
            let msg = commit::service::format_message(&entry);

            if total > 1 {
                println!(
                    "\n{}",
                    color::bold_cyan(&format!("--- 提交 {}/{} ---", i + 1, total))
                );
            }

            if ctx.auto_commit {
                commit::service::print_entry(&entry);
            } else if !commit::service::confirm_entry(&entry) {
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
