use std::collections::HashMap;
use std::sync::Arc;

use crate::config::AiConfig;

pub mod openai;
pub mod prompt;

#[derive(Debug, Clone)]
pub struct Request {
    pub types: Vec<String>,
    pub scopes: Vec<String>,
    pub diff: String,
    pub git_info: String,
    pub extra_context: String,
}

pub trait AiProvider: Send + Sync {
    fn generate(&self, req: &Request) -> anyhow::Result<String>;
}

pub type ProviderFactory = fn(AiConfig) -> anyhow::Result<Arc<dyn AiProvider>>;

/// 策略注册表：按名称分发到具体 provider 实现
pub struct ProviderRegistry {
    factories: HashMap<String, ProviderFactory>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };
        registry.register("openai", openai::OpenAI::create);
        registry
    }

    pub fn register(&mut self, name: &str, factory: ProviderFactory) {
        self.factories.insert(name.to_string(), factory);
    }

    pub fn create(&self, name: &str, config: AiConfig) -> anyhow::Result<Arc<dyn AiProvider>> {
        let factory = self
            .factories
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("未知的 AI provider: {name}"))?;
        factory(config)
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider;

    impl AiProvider for MockProvider {
        fn generate(&self, _req: &Request) -> anyhow::Result<String> {
            Ok("mock".into())
        }
    }

    fn mock_factory(_config: AiConfig) -> anyhow::Result<Arc<dyn AiProvider>> {
        Ok(Arc::new(MockProvider))
    }

    #[test]
    fn create_dispatches_to_registered_factory() {
        let mut registry = ProviderRegistry::new();
        registry.register("mock", mock_factory);

        let provider = registry
            .create("mock", AiConfig {
                provider: "mock".into(),
                api_key: None,
                model: "m".into(),
                base_url: "u".into(),
            })
            .unwrap();
        assert_eq!(
            provider.generate(&Request {
                types: vec![],
                scopes: vec![],
                diff: String::new(),
                git_info: String::new(),
                extra_context: String::new(),
            }).unwrap(),
            "mock"
        );
    }

    #[test]
    fn create_unknown_provider_errors() {
        let registry = ProviderRegistry::new();
        let result = registry.create("unknown", AiConfig {
            provider: "unknown".into(),
            api_key: None,
            model: "m".into(),
            base_url: "u".into(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn default_registry_has_openai() {
        let registry = ProviderRegistry::new();
        assert!(registry.factories.contains_key("openai"));
    }
}
