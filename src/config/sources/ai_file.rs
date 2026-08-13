use std::fs;
use std::path::PathBuf;

use crate::config::paths::config_dir_default;
use crate::config::source::ConfigSource;
use crate::config::{FileConfig, ProviderSetting};

const DEFAULT_CONFIG_FILE: &str = "default.yaml";
/// 从配置目录的 default.yaml 加载 AI 配置。
/// 文件不存在时静默返回 None（回退链正常路径）；
/// 文件存在但解析失败时输出可读警告，避免静默吞错。
pub struct AiFileSource {
    dir: Option<PathBuf>,
}

impl AiFileSource {
    pub fn new(config_dir: Option<String>) -> Self {
        Self {
            dir: config_dir
                .filter(|d| !d.is_empty())
                .map(PathBuf::from)
                .or_else(config_dir_default),
        }
    }

    pub fn config_path(&self) -> Option<PathBuf> {
        self.dir.as_ref().map(|d| d.join(DEFAULT_CONFIG_FILE))
    }
}

impl ConfigSource<FileConfig> for AiFileSource {
    fn load(&self) -> Option<FileConfig> {
        let path = self.config_path()?;
        if !path.exists() {
            return None;
        }
        let data = fs::read_to_string(&path).ok()?;
        match serde_yml::from_str::<FileConfig>(&data) {
            Ok(cfg) => Some(normalize(cfg)),
            Err(e) => {
                eprintln!(
                    "{}",
                    crate::ui::color::yellow(&format!(
                        "警告: 配置文件 {} 解析失败，已跳过该配置源: {e}",
                        path.display()
                    ))
                );
                None
            }
        }
    }
}

/// 兼容旧扁平结构：providers 为空但存在扁平字段时，归一化为单个 provider
fn normalize(mut cfg: FileConfig) -> FileConfig {
    if cfg.providers.is_empty() {
        if let Some(name) = cfg.provider.clone() {
            cfg.providers.push(ProviderSetting {
                name,
                kind: None,
                api_key: cfg.api_key.clone(),
                model: cfg.model.clone(),
                base_url: cfg.base_url.clone(),
            });
            if cfg.use_provider.is_none() {
                cfg.use_provider = cfg.provider.clone();
            }
        }
    }
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir_with(content: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(DEFAULT_CONFIG_FILE), content).unwrap();
        dir
    }

    #[test]
    fn load_reads_providers_list() {
        let dir = temp_dir_with(
            "use_provider: deepseek\nproviders:\n  - name: deepseek\n    api_key: file-key\n",
        );
        let source = AiFileSource::new(Some(dir.path().to_string_lossy().to_string()));

        let cfg = source.load().unwrap();
        assert_eq!(cfg.use_provider.as_deref(), Some("deepseek"));
        assert_eq!(cfg.providers.len(), 1);
        assert_eq!(cfg.providers[0].name, "deepseek");
        assert_eq!(cfg.providers[0].api_key.as_deref(), Some("file-key"));
    }

    #[test]
    fn load_normalizes_legacy_flat_structure() {
        let dir = temp_dir_with("provider: custom\napi_key: file-key\n");
        let source = AiFileSource::new(Some(dir.path().to_string_lossy().to_string()));

        let cfg = source.load().unwrap();
        assert_eq!(cfg.providers.len(), 1);
        assert_eq!(cfg.providers[0].name, "custom");
        assert_eq!(cfg.providers[0].api_key.as_deref(), Some("file-key"));
        assert_eq!(cfg.use_provider.as_deref(), Some("custom"));
    }

    #[test]
    fn load_missing_dir_returns_none() {
        let source = AiFileSource::new(Some("C:/nonexistent/agc-dir".into()));
        assert!(source.load().is_none());
    }

    #[test]
    fn load_missing_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let source = AiFileSource::new(Some(dir.path().to_string_lossy().to_string()));
        assert!(source.load().is_none());
    }

    #[test]
    fn load_corrupted_yaml_returns_none() {
        let dir = temp_dir_with("provider: [unclosed");
        let source = AiFileSource::new(Some(dir.path().to_string_lossy().to_string()));
        assert!(source.load().is_none());
    }

    #[test]
    fn config_path_joins_default_yaml() {
        let source = AiFileSource::new(Some("C:/custom/dir".into()));
        assert_eq!(
            source.config_path().unwrap(),
            std::path::Path::new("C:/custom/dir").join(DEFAULT_CONFIG_FILE)
        );
    }
}
