use crate::commit::service;
use crate::pipeline::{PipelineContext, PipelineStage};
use crate::ui::color;

pub struct EntryParser;

impl PipelineStage for EntryParser {
    fn execute(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        if !ctx.has_changes {
            return Ok(());
        }
        if ctx.raw_response.is_empty() {
            anyhow::bail!("AI 未返回任何内容");
        }

        let (entries, reason) = service::parse_entries(&ctx.raw_response);
        if entries.is_empty() {
            anyhow::bail!("AI 未生成有效 commit 消息");
        }
        if !reason.is_empty() {
            println!("\n{}", color::yellow(&reason));
        }

        ctx.entries = entries;
        ctx.reason = reason;
        Ok(())
    }
}
