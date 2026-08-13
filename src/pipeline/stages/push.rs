use std::sync::Arc;

use crate::git::GitBackend;
use crate::pipeline::{PipelineContext, PipelineStage};
use crate::ui::color;
use crate::ui::spinner;

pub struct GitPusher {
    git: Arc<dyn GitBackend>,
    remote: String,
    branch: String,
}

impl GitPusher {
    pub fn new(git: Arc<dyn GitBackend>, remote: String, branch: String) -> Self {
        Self { git, remote, branch }
    }
}

impl PipelineStage for GitPusher {
    fn execute(&self, _ctx: &mut PipelineContext) -> anyhow::Result<()> {
        let target = format!("{} {}", self.remote, self.branch);
        spinner::run(&format!("正在执行 git push {target}"), || {
            self.git.push(&self.remote, &self.branch)
        })?;
        println!("{}", color::green(&format!("git push {target} 成功")));
        Ok(())
    }
}
