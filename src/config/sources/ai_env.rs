use crate::config::source::ConfigSource;
use crate::config::{FileConfig, ProviderSetting};

/// 从环境变量加载 AI 配置（向后兼容旧变量名），
/// 生成单个 provider（name 取 AI_PROVIDER，默认 openai）
pub struct AiEnvSource;

impl ConfigSource<FileConfig> for AiEnvSource {
    fn load(&self) -> Option<FileConfig> {
        let api_key = std::env::var("MESSAGE_API_KEY").ok();
        let provider = std::env::var("AI_PROVIDER").ok();
        let model = std::env::var("OPENAI_MODEL").ok();
        let base_url = std::env::var("OPENAI_BASE_URL").ok();

        if api_key.is_none() && provider.is_none() && model.is_none() && base_url.is_none() {
            return None;
        }
        Some(FileConfig {
            use_provider: provider.clone(),
            providers: vec![ProviderSetting {
                name: provider.unwrap_or_else(|| "openai".to_string()),
                kind: None,
                api_key,
                model,
                base_url,
            }],
            provider: None,
            api_key: None,
            model: None,
            base_url: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_env::{self, restore_var};

    #[test]
    fn load_reads_env_vars() {
        let _guard = test_env::LOCK.lock().unwrap();

        let orig_key = std::env::var("MESSAGE_API_KEY").ok();
        let orig_provider = std::env::var("AI_PROVIDER").ok();
        std::env::set_var("MESSAGE_API_KEY", "env-key");
        std::env::set_var("AI_PROVIDER", "env-provider");

        let cfg = AiEnvSource.load().unwrap();
        assert_eq!(cfg.providers.len(), 1);
        assert_eq!(cfg.providers[0].name, "env-provider");
        assert_eq!(cfg.providers[0].api_key.as_deref(), Some("env-key"));
        assert!(cfg.providers[0].model.is_none());

        restore_var("MESSAGE_API_KEY", orig_key);
        restore_var("AI_PROVIDER", orig_provider);
    }

    #[test]
    fn load_default_provider_name_is_openai() {
        let _guard = test_env::LOCK.lock().unwrap();

        let orig_key = std::env::var("MESSAGE_API_KEY").ok();
        std::env::set_var("MESSAGE_API_KEY", "env-key");

        let cfg = AiEnvSource.load().unwrap();
        assert_eq!(cfg.providers[0].name, "openai");

        restore_var("MESSAGE_API_KEY", orig_key);
    }

    #[test]
    fn load_no_env_returns_none() {
        let _guard = test_env::LOCK.lock().unwrap();

        let orig_key = std::env::var("MESSAGE_API_KEY").ok();
        let orig_provider = std::env::var("AI_PROVIDER").ok();
        let orig_model = std::env::var("OPENAI_MODEL").ok();
        let orig_url = std::env::var("OPENAI_BASE_URL").ok();
        std::env::remove_var("MESSAGE_API_KEY");
        std::env::remove_var("AI_PROVIDER");
        std::env::remove_var("OPENAI_MODEL");
        std::env::remove_var("OPENAI_BASE_URL");

        assert!(AiEnvSource.load().is_none());

        restore_var("MESSAGE_API_KEY", orig_key);
        restore_var("AI_PROVIDER", orig_provider);
        restore_var("OPENAI_MODEL", orig_model);
        restore_var("OPENAI_BASE_URL", orig_url);
    }
}
