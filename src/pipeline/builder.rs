use std::sync::Arc;

use crate::ai::AiProvider;
use crate::config::Config;
use crate::git::GitBackend;
use crate::search::SearchBackend;

use super::stages::*;
use super::{Pipeline, PipelineStage};

/// 建造者模式：构造管线及其所有阶段
pub struct PipelineBuilder {
    git: Option<Arc<dyn GitBackend>>,
    ai: Option<Arc<dyn AiProvider>>,
    search: Option<Arc<dyn SearchBackend>>,
    config: Option<Config>,
    rg_pattern: String,
    fd_pattern: String,
    auto_push: Option<String>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            git: None,
            ai: None,
            search: None,
            config: None,
            rg_pattern: String::new(),
            fd_pattern: String::new(),
            auto_push: None,
        }
    }

    pub fn with_git(mut self, git: Arc<dyn GitBackend>) -> Self {
        self.git = Some(git);
        self
    }

    pub fn with_ai(mut self, ai: Arc<dyn AiProvider>) -> Self {
        self.ai = Some(ai);
        self
    }

    pub fn with_search(mut self, search: Arc<dyn SearchBackend>) -> Self {
        self.search = Some(search);
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

    pub fn with_auto_push(mut self, remote: Option<String>) -> Self {
        self.auto_push = remote;
        self
    }

    pub fn build(self) -> anyhow::Result<Pipeline> {
        let git = self
            .git
            .ok_or_else(|| anyhow::anyhow!("GitBackend 未设置"))?;
        let ai = self
            .ai
            .ok_or_else(|| anyhow::anyhow!("AiProvider 未设置"))?;
        let search = self
            .search
            .ok_or_else(|| anyhow::anyhow!("SearchBackend 未设置"))?;
        let config = self
            .config
            .ok_or_else(|| anyhow::anyhow!("Config 未设置"))?;

        let mut stages: Vec<Box<dyn PipelineStage>> = vec![
            Box::new(DiffCollector::new(Arc::clone(&git))),
            Box::new(ContextEnricher::new(
                Arc::clone(&search),
                self.rg_pattern,
                self.fd_pattern,
            )),
            Box::new(AiGenerator::new(Arc::clone(&ai), config)),
            Box::new(EntryParser),
            Box::new(CommitExecutor::new(Arc::clone(&git))),
        ];

        if let Some(remote) = self.auto_push {
            stages.push(Box::new(GitPusher::new(git, remote)));
        }

        Ok(Pipeline::new(stages))
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
