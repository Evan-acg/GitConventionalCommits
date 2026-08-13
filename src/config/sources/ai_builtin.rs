use crate::config::source::ConfigSource;
use crate::config::{FileConfig, ProviderSetting};

pub const DEFAULT_PROVIDER: &str = "openai";
pub const DEFAULT_MODEL: &str = "deepseek-v4-flash";
/// 完整 chat/completions 端点地址（不再由代码拼接）
pub const DEFAULT_BASE_URL: &str = "https://api.deepseek.com/v1/chat/completions";

/// 内置默认 AI 配置（兜底源，始终返回 Some）
pub struct AiBuiltinSource;

impl ConfigSource<FileConfig> for AiBuiltinSource {
    fn load(&self) -> Option<FileConfig> {
        Some(FileConfig {
            use_provider: Some(DEFAULT_PROVIDER.into()),
            providers: vec![ProviderSetting {
                name: DEFAULT_PROVIDER.into(),
                kind: Some(DEFAULT_PROVIDER.into()),
                api_key: None,
                model: Some(DEFAULT_MODEL.into()),
                base_url: Some(DEFAULT_BASE_URL.into()),
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

    #[test]
    fn load_returns_builtin_defaults() {
        let cfg = AiBuiltinSource.load().unwrap();
        assert_eq!(cfg.use_provider.as_deref(), Some(DEFAULT_PROVIDER));
        assert_eq!(cfg.providers.len(), 1);
        assert_eq!(cfg.providers[0].name, DEFAULT_PROVIDER);
        assert_eq!(cfg.providers[0].kind.as_deref(), Some(DEFAULT_PROVIDER));
        assert_eq!(cfg.providers[0].model.as_deref(), Some(DEFAULT_MODEL));
        assert_eq!(cfg.providers[0].base_url.as_deref(), Some(DEFAULT_BASE_URL));
        assert!(cfg.providers[0].api_key.is_none());
    }
}
