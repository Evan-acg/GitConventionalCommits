use std::sync::Arc;

use crate::git::GitBackend;
use crate::pipeline::{PipelineContext, PipelineStage};
use crate::ui::color;
use crate::ui::spinner;

pub struct GitPusher {
    git: Arc<dyn GitBackend>,
    remote: String,
}

impl GitPusher {
    pub fn new(git: Arc<dyn GitBackend>, remote: String) -> Self {
        Self { git, remote }
    }
}

impl PipelineStage for GitPusher {
    fn execute(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        if !ctx.has_changes {
            return Ok(());
        }

        spinner::run(&format!("正在执行 git push {}", self.remote), || {
            self.git.push(&self.remote)
        })?;
        println!("{}", color::green(&format!("git push {} 成功", self.remote)));
        Ok(())
    }
}
