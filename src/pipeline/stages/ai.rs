use std::sync::Arc;

use crate::ai::{AiProvider, Request};
use crate::config::Config;
use crate::pipeline::{PipelineContext, PipelineStage};
use crate::ui::spinner;

pub struct AiGenerator {
    ai: Arc<dyn AiProvider>,
    config: Config,
}

impl AiGenerator {
    pub fn new(ai: Arc<dyn AiProvider>, config: Config) -> Self {
        Self { ai, config }
    }
}

impl PipelineStage for AiGenerator {
    fn execute(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        if !ctx.has_changes {
            return Ok(());
        }

        spinner::run("正在调用 AI 生成 commit 消息", || {
            let r = self.ai.generate(&Request {
                types: self.config.types.clone(),
                scopes: self.config.scopes.clone(),
                diff: ctx.diff.clone(),
                git_info: ctx.git_info.clone(),
                extra_context: ctx.extra_context.clone(),
            })?;
            ctx.raw_response = r;
            Ok(())
        })
    }
}
