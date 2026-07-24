use std::sync::Arc;

use crate::pipeline::{PipelineContext, PipelineStage};
use crate::search::{merge_context, SearchBackend};

pub struct ContextEnricher {
    search: Arc<dyn SearchBackend>,
    rg_pattern: String,
    fd_pattern: String,
}

impl ContextEnricher {
    pub fn new(search: Arc<dyn SearchBackend>, rg_pattern: String, fd_pattern: String) -> Self {
        Self {
            search,
            rg_pattern,
            fd_pattern,
        }
    }
}

impl PipelineStage for ContextEnricher {
    fn execute(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        if !ctx.has_changes {
            return Ok(());
        }

        let rg = self.search.rg_context(&ctx.changed_files, &self.rg_pattern);
        let fd = self.search.fd_context(&ctx.changed_files, &self.fd_pattern);
        ctx.extra_context = merge_context(&rg, &fd);
        Ok(())
    }
}
