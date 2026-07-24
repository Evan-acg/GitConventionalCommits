use std::sync::Arc;

use crate::git::GitBackend;
use crate::pipeline::{PipelineContext, PipelineStage};
use crate::ui::spinner;

pub struct DiffCollector {
    git: Arc<dyn GitBackend>,
}

impl DiffCollector {
    pub fn new(git: Arc<dyn GitBackend>) -> Self {
        Self { git }
    }
}

impl PipelineStage for DiffCollector {
    fn execute(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        spinner::run("正在获取 git 变更信息", || {
            ctx.diff = self.git.diff()?;
            ctx.changed_files = self.git.changed_files().unwrap_or_default();
            ctx.status_short = self.git.status_short().unwrap_or_default();
            ctx.diff_stat = self.git.diff_stat().unwrap_or_default();
            ctx.diff_cached_stat = self.git.diff_cached_stat().unwrap_or_default();
            ctx.untracked_files = self.git.ls_untracked().unwrap_or_default();
            Ok(())
        })?;

        ctx.has_changes = !ctx.diff.is_empty() || !ctx.untracked_files.is_empty();
        if !ctx.has_changes {
            println!("没有未暂存的变更");
            return Ok(());
        }

        ctx.build_git_info();
        Ok(())
    }
}
