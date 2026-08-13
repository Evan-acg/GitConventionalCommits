pub mod ai;
pub mod init;
pub mod paths;
pub mod source;
pub mod sources;

pub use ai::{AiConfig, FileConfig, ProviderSetting};
pub use source::{ConfigChain, ConfigSource};

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub types: Vec<String>,
    pub scopes: Vec<String>,
}

impl Config {
    pub fn is_empty(&self) -> bool {
        self.types.is_empty() && self.scopes.is_empty()
    }
}

/// 应用配置门面（Facade）：聚合项目所有配置域，屏蔽内部加载链细节
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub types: Vec<String>,
    pub scopes: Vec<String>,
    pub ai: AiConfig,
}

impl AppConfig {
    pub fn load(
        yaml_path: &str,
        markdown_path: &str,
        config_dir: Option<String>,
        api_key: Option<String>,
    ) -> anyhow::Result<Self> {
        let base = default_chain(yaml_path, markdown_path)
            .first()
            .unwrap_or_default();
        let ai = ai::AiConfigLoader::new()
            .with_config_dir(config_dir)
            .with_api_key(api_key)
            .build()?;

        Ok(Self {
            types: base.types,
            scopes: base.scopes,
            ai,
        })
    }
}

/// 标准 type/scope 配置链：.lazygit.yaml > SKILL.md > 内置默认值
pub fn default_chain(yaml_path: &str, markdown_path: &str) -> ConfigChain<Config> {
    ConfigChain::new(vec![
        Box::new(sources::YamlSource::new(yaml_path)),
        Box::new(sources::MarkdownSource::new(markdown_path)),
        Box::new(sources::DefaultsSource),
    ])
}

#[cfg(test)]
pub mod test_env {
    use std::sync::Mutex;

    /// 串行化所有读写环境变量的单测，避免并行执行时互相干扰
    pub static LOCK: Mutex<()> = Mutex::new(());

    /// 恢复环境变量原值（原值为 None 时删除）
    pub fn restore_var(name: &str, value: Option<String>) {
        match value {
            Some(v) => std::env::set_var(name, v),
            None => std::env::remove_var(name),
        }
    }
}
