mod builder;
mod context;
pub mod stages;

pub use builder::PipelineBuilder;
pub use context::PipelineContext;

/// 管线阶段：对上下文执行一次转换
pub trait PipelineStage: Send + Sync {
    fn execute(&self, ctx: &mut PipelineContext) -> anyhow::Result<()>;
}

/// 管线：按序执行多个阶段
pub struct Pipeline {
    stages: Vec<Box<dyn PipelineStage>>,
}

impl Pipeline {
    pub fn new(stages: Vec<Box<dyn PipelineStage>>) -> Self {
        Self { stages }
    }

    pub fn run(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        for stage in &self.stages {
            stage.execute(ctx)?;
        }
        Ok(())
    }
}
