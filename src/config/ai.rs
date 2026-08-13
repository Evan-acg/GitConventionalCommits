use serde::Deserialize;

use crate::config::source::{ConfigChain, Mergeable};
use crate::config::sources::ai_builtin::{DEFAULT_BASE_URL, DEFAULT_MODEL, DEFAULT_PROVIDER};
use crate::config::sources::{AiBuiltinSource, AiEnvSource, AiFileSource};

/// 部分 AI 配置（所有字段可选），作为各配置源之间的传递载体
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileConfig {
    #[serde(default)]
    pub use_provider: Option<String>,
    #[serde(default)]
    pub providers: Vec<ProviderSetting>,
    /// 旧扁平结构（兼容）：providers 为空时归一化为单个 provider
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
}

impl Mergeable for FileConfig {
    fn merge_from(&mut self, higher: Self) {
        if higher.use_provider.is_some() {
            self.use_provider = higher.use_provider;
        }
        for hp in higher.providers {
            match self.providers.iter_mut().find(|p| p.name == hp.name) {
                Some(lp) => lp.merge_from(hp),
                None => self.providers.push(hp),
            }
        }
    }
}

/// 单个连接配置
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderSetting {
    pub name: String,
    /// 协议实现类型（ProviderRegistry 注册名），默认 openai
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    /// 完整 chat/completions 端点地址
    #[serde(default)]
    pub base_url: Option<String>,
}

impl Mergeable for ProviderSetting {
    fn merge_from(&mut self, higher: Self) {
        if higher.kind.is_some() {
            self.kind = higher.kind;
        }
        if higher.api_key.is_some() {
            self.api_key = higher.api_key;
        }
        if higher.model.is_some() {
            self.model = higher.model;
        }
        if higher.base_url.is_some() {
            self.base_url = higher.base_url;
        }
    }
}

/// 最终解析完成的 AI 配置
#[derive(Debug, Clone)]
pub struct AiConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: String,
}

/// AI 配置加载器（Builder）：按优先级组装配置源，
/// 配置文件 > 环境变量 > 内置默认值，再按 use_provider 选中连接。
pub struct AiConfigLoader {
    config_dir: Option<String>,
    api_key: Option<String>,
}

impl AiConfigLoader {
    pub fn new() -> Self {
        Self {
            config_dir: None,
            api_key: None,
        }
    }

    pub fn with_config_dir(mut self, config_dir: Option<String>) -> Self {
        self.config_dir = config_dir;
        self
    }

    pub fn with_api_key(mut self, api_key: Option<String>) -> Self {
        self.api_key = api_key;
        self
    }

    pub fn build(self) -> anyhow::Result<AiConfig> {
        let chain = ConfigChain::new(vec![
            Box::new(AiFileSource::new(self.config_dir)),
            Box::new(AiEnvSource),
            Box::new(AiBuiltinSource),
        ]);

        let merged = chain.merge().unwrap_or_default();

        let selected = merged
            .use_provider
            .as_deref()
            .unwrap_or(DEFAULT_PROVIDER);
        let setting = merged
            .providers
            .iter()
            .find(|p| p.name == selected)
            .ok_or_else(|| {
                let available = merged
                    .providers
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                anyhow::anyhow!(
                    "use_provider '{selected}' 未在 providers 中定义，可用配置: {}",
                    if available.is_empty() { "(无)" } else { &available }
                )
            })?;

        let api_key = self
            .api_key
            .filter(|k| !k.is_empty())
            .or_else(|| setting.api_key.clone())
            .or_else(|| std::env::var("MESSAGE_API_KEY").ok());

        Ok(AiConfig {
            provider: setting
                .kind
                .as_deref()
                .unwrap_or(DEFAULT_PROVIDER)
                .to_string(),
            api_key,
            model: setting
                .model
                .clone()
                .unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            base_url: setting
                .base_url
                .clone()
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
        })
    }
}

impl Default for AiConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_default_yaml(dir: &tempfile::TempDir, content: &str) {
        fs::write(dir.path().join("default.yaml"), content).unwrap();
    }

    #[test]
    fn build_selects_provider_by_use_provider() {
        let dir = tempfile::tempdir().unwrap();
        write_default_yaml(
            &dir,
            r#"
use_provider: deepseek
providers:
  - name: deepseek
    base_url: https://api.deepseek.com/v1/chat/completions
    model: deepseek-v4-flash
    api_key: file-key
  - name: opencode-go
    base_url: https://opencode.ai/zen/go/v1/chat/completions
    model: deepseek-v4-flash
    api_key: other-key
"#,
        );

        let cfg = AiConfigLoader::new()
            .with_config_dir(Some(dir.path().to_string_lossy().to_string()))
            .build()
            .unwrap();
        assert_eq!(cfg.provider, "openai");
        assert_eq!(cfg.api_key.as_deref(), Some("file-key"));
        assert_eq!(cfg.model, "deepseek-v4-flash");
        assert_eq!(cfg.base_url, "https://api.deepseek.com/v1/chat/completions");
    }

    #[test]
    fn build_uses_kind_as_provider_impl() {
        let dir = tempfile::tempdir().unwrap();
        write_default_yaml(
            &dir,
            r#"
use_provider: custom
providers:
  - name: custom
    kind: anthropic
    base_url: https://example.com/v1
"#,
        );

        let cfg = AiConfigLoader::new()
            .with_config_dir(Some(dir.path().to_string_lossy().to_string()))
            .build()
            .unwrap();
        assert_eq!(cfg.provider, "anthropic");
    }

    #[test]
    fn build_cli_api_key_overrides_selected() {
        let dir = tempfile::tempdir().unwrap();
        write_default_yaml(
            &dir,
            "use_provider: deepseek\nproviders:\n  - name: deepseek\n    api_key: file-key\n",
        );

        let cfg = AiConfigLoader::new()
            .with_config_dir(Some(dir.path().to_string_lossy().to_string()))
            .with_api_key(Some("cli-key".into()))
            .build()
            .unwrap();
        assert_eq!(cfg.api_key.as_deref(), Some("cli-key"));
    }

    #[test]
    fn build_unknown_use_provider_errors() {
        let dir = tempfile::tempdir().unwrap();
        write_default_yaml(
            &dir,
            "use_provider: ghost\nproviders:\n  - name: deepseek\n",
        );

        let err = AiConfigLoader::new()
            .with_config_dir(Some(dir.path().to_string_lossy().to_string()))
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("ghost"));
        assert!(err.to_string().contains("deepseek"));
    }

    #[test]
    fn build_legacy_flat_config_normalized() {
        let dir = tempfile::tempdir().unwrap();
        write_default_yaml(&dir, "provider: openai\napi_key: flat-key\n");

        let cfg = AiConfigLoader::new()
            .with_config_dir(Some(dir.path().to_string_lossy().to_string()))
            .build()
            .unwrap();
        assert_eq!(cfg.api_key.as_deref(), Some("flat-key"));
    }

    #[test]
    fn build_falls_back_to_env_and_defaults() {
        use crate::config::test_env::{self, restore_var};

        let _guard = test_env::LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();

        let orig_key = std::env::var("MESSAGE_API_KEY").ok();
        let orig_provider = std::env::var("AI_PROVIDER").ok();
        std::env::set_var("MESSAGE_API_KEY", "env-key");
        std::env::set_var("AI_PROVIDER", "env-provider");

        let cfg = AiConfigLoader::new()
            .with_config_dir(Some(dir.path().to_string_lossy().to_string()))
            .build()
            .unwrap();
        assert_eq!(cfg.api_key.as_deref(), Some("env-key"));
        assert_eq!(cfg.provider, "openai");
        assert_eq!(cfg.model, DEFAULT_MODEL);
        assert_eq!(cfg.base_url, DEFAULT_BASE_URL);

        restore_var("MESSAGE_API_KEY", orig_key);
        restore_var("AI_PROVIDER", orig_provider);
    }

    #[test]
    fn build_missing_dir_uses_defaults() {
        let cfg = AiConfigLoader::new()
            .with_config_dir(Some("C:/nonexistent/agc-dir".into()))
            .build()
            .unwrap();
        assert_eq!(cfg.provider, DEFAULT_PROVIDER);
        assert_eq!(cfg.model, DEFAULT_MODEL);
        assert_eq!(cfg.base_url, DEFAULT_BASE_URL);
    }
}
