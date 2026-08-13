use super::super::source::ConfigSource;
use super::super::Config;

/// 内置默认 type/scope 配置（兜底源，始终返回 Some）
pub struct DefaultsSource;

impl ConfigSource<Config> for DefaultsSource {
    fn load(&self) -> Option<Config> {
        Some(Config {
            types: vec![
                "Feat".into(),
                "Fix".into(),
                "Docs".into(),
                "Style".into(),
                "Refactor".into(),
                "Perf".into(),
                "Test".into(),
                "Build".into(),
                "CI".into(),
                "Chore".into(),
                "Revert".into(),
            ],
            scopes: vec![],
        })
    }
}
