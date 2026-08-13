use std::fs;
use std::path::PathBuf;

use crate::config::paths::config_dir_default;

const CONFIG_FILE: &str = "default.yaml";

const DEFAULT_YAML_TEMPLATE: &str = r#"# agc AI 配置
# use_provider 选择 providers 列表中要使用的连接配置
use_provider: deepseek
providers:
  - name: deepseek
    # kind 为协议实现类型（ProviderRegistry 注册名），默认 openai
    kind: openai
    # base_url 为完整 chat/completions 端点地址，代码不再自动拼接
    base_url: https://api.deepseek.com/v1/chat/completions
    model: deepseek-v4-flash
    # api_key 可省略，省略时回退到 MESSAGE_API_KEY 环境变量
    api_key: sk-xxx
"#;

pub fn resolve_dir(path: Option<String>) -> PathBuf {
    match path.filter(|p| !p.is_empty()) {
        Some(p) => PathBuf::from(p),
        None => config_dir_default().unwrap_or_else(|| PathBuf::from(".config/agc")),
    }
}

pub fn run(path: Option<String>, force: bool) -> anyhow::Result<()> {
    let dir = resolve_dir(path);
    let file = dir.join(CONFIG_FILE);

    if file.exists() && !force {
        anyhow::bail!(
            "{} 已存在，如需覆盖请添加 --force",
            file.display()
        );
    }

    fs::create_dir_all(&dir)
        .map_err(|e| anyhow::anyhow!("无法创建配置目录 {}: {e}", dir.display()))?;
    fs::write(&file, DEFAULT_YAML_TEMPLATE)?;
    println!("已生成配置文件: {}", file.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_creates_default_yaml_in_given_dir() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();

        run(Some(path.clone()), false).unwrap();

        let content = fs::read_to_string(dir.path().join("default.yaml")).unwrap();
        assert!(content.contains("use_provider: deepseek"));
        assert!(content.contains("providers:"));
    }

    #[test]
    fn run_creates_missing_nested_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a/b/c").to_string_lossy().to_string();

        run(Some(path), false).unwrap();

        assert!(dir.path().join("a/b/c/default.yaml").exists());
    }

    #[test]
    fn run_refuses_overwrite_without_force() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();
        fs::write(dir.path().join("default.yaml"), "existing").unwrap();

        let err = run(Some(path.clone()), false).unwrap_err();
        assert!(err.to_string().contains("已存在"));
        assert_eq!(fs::read_to_string(dir.path().join("default.yaml")).unwrap(), "existing");
    }

    #[test]
    fn run_overwrites_with_force() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();
        fs::write(dir.path().join("default.yaml"), "existing").unwrap();

        run(Some(path.clone()), true).unwrap();

        let content = fs::read_to_string(dir.path().join("default.yaml")).unwrap();
        assert!(content.contains("use_provider: deepseek"));
    }

    #[test]
    fn resolve_dir_uses_given_path() {
        assert_eq!(
            resolve_dir(Some("C:/custom/dir".into())),
            PathBuf::from("C:/custom/dir")
        );
    }

    #[test]
    fn resolve_dir_empty_falls_back_to_default() {
        assert_eq!(resolve_dir(Some(String::new())), config_dir_default().unwrap());
        assert_eq!(resolve_dir(None), config_dir_default().unwrap());
    }

    #[cfg(not(target_family = "unix"))]
    #[test]
    fn run_without_path_creates_default_dirs() {
        use crate::config::test_env::{self, restore_var};

        let _guard = test_env::LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let orig_user = std::env::var("USERPROFILE").ok();
        let orig_app = std::env::var("APPDATA").ok();
        std::env::set_var("USERPROFILE", temp.path());
        std::env::set_var("APPDATA", "");

        run(None, false).unwrap();

        let file = temp.path().join(".config").join("agc").join("default.yaml");
        assert!(file.exists(), "default config should be created at {file:?}");

        restore_var("USERPROFILE", orig_user);
        restore_var("APPDATA", orig_app);
    }
}
