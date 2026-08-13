use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const PROJECT_DIR: &str = ".project";
pub const SCOPE_FILE: &str = ".git-style-scope.yaml";

const FILE_HEADER: &str = "# 项目 type/scope 定义（与 .lazygit.yaml 同格式）\n";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeYaml {
    #[serde(rename = "type", default)]
    pub types: Vec<TypeScopeItem>,
    #[serde(default)]
    pub scopes: Vec<TypeScopeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeScopeItem {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

pub fn scope_file(base: &Path) -> PathBuf {
    base.join(PROJECT_DIR).join(SCOPE_FILE)
}

/// 家目录：unix 用 $HOME，Windows 用 $USERPROFILE
pub fn home_dir() -> Option<PathBuf> {
    #[cfg(target_family = "unix")]
    {
        std::env::var("HOME")
            .ok()
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    }
    #[cfg(not(target_family = "unix"))]
    {
        std::env::var("USERPROFILE")
            .ok()
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    }
}

/// 查找链（优先级 1 > 2 > 3）：.project 目录 > 项目根目录 > 家目录
pub fn chain_paths(base: &Path) -> Vec<PathBuf> {
    let mut paths = vec![scope_file(base), base.join(SCOPE_FILE)];
    if let Some(home) = home_dir() {
        let home_file = home.join(SCOPE_FILE);
        if !paths.contains(&home_file) {
            paths.push(home_file);
        }
    }
    paths
}

/// 返回查找链上第一个存在的文件
pub fn resolve_scope_path(base: &Path) -> Option<PathBuf> {
    chain_paths(base).into_iter().find(|p| p.is_file())
}

/// 按查找链读取（1 > 2 > 3）
pub fn read_scope_yaml(base: &Path) -> Option<ScopeYaml> {
    let path = resolve_scope_path(base)?;
    read_scope_yaml_at(&path)
}

pub fn read_scope_yaml_at(path: &Path) -> Option<ScopeYaml> {
    let data = fs::read_to_string(path).ok()?;
    serde_yml::from_str(&data).ok()
}

pub fn serialize(yaml: &ScopeYaml) -> String {
    serde_yml::to_string(yaml).unwrap_or_default()
}

pub fn write_scope_yaml(base: &Path, yaml: &ScopeYaml) -> anyhow::Result<()> {
    let dir = base.join(PROJECT_DIR);
    fs::create_dir_all(&dir)
        .map_err(|e| anyhow::anyhow!("无法创建目录 {}: {e}", dir.display()))?;
    fs::write(scope_file(base), format!("{FILE_HEADER}{}", serialize(yaml)))?;
    Ok(())
}

/// 默认 type 列表（与 init 模板一致），用于文件缺失时自动创建
pub fn default_types() -> Vec<TypeScopeItem> {
    vec![
        TypeScopeItem { name: "Feat".into(), docs: Some("新特性".into()) },
        TypeScopeItem { name: "Fix".into(), docs: Some("修复缺陷".into()) },
        TypeScopeItem { name: "Docs".into(), docs: Some("文档变更".into()) },
        TypeScopeItem { name: "Refactor".into(), docs: Some("重构代码".into()) },
        TypeScopeItem { name: "Test".into(), docs: Some("测试相关".into()) },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let yaml = ScopeYaml {
            types: default_types(),
            scopes: vec![TypeScopeItem {
                name: "Pipeline".into(),
                docs: Some("管线".into()),
            }],
        };

        write_scope_yaml(dir.path(), &yaml).unwrap();
        let loaded = read_scope_yaml(dir.path()).unwrap();
        assert_eq!(loaded.types.len(), 5);
        assert_eq!(loaded.scopes[0].name, "Pipeline");
        assert_eq!(loaded.scopes[0].docs.as_deref(), Some("管线"));
    }

    #[test]
    fn read_missing_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_scope_yaml(dir.path()).is_none());
    }

    #[test]
    fn read_ignores_comments_and_empty() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(PROJECT_DIR)).unwrap();
        fs::write(
            scope_file(dir.path()),
            "# 注释\nscopes:\n  - name: Core\n",
        )
        .unwrap();
        let loaded = read_scope_yaml(dir.path()).unwrap();
        assert_eq!(loaded.scopes[0].name, "Core");
    }

    fn write_at(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[cfg(target_family = "unix")]
    fn home_env_var() -> &'static str {
        "HOME"
    }

    #[cfg(not(target_family = "unix"))]
    fn home_env_var() -> &'static str {
        "USERPROFILE"
    }

    /// 在家目录可见的场景下串行执行（env 隔离）
    fn with_temp_home(dir: &Path, f: impl FnOnce()) {
        let _guard = crate::config::test_env::LOCK.lock().unwrap();
        let var = home_env_var();
        let orig = std::env::var(var).ok();
        std::env::set_var(var, dir);
        f();
        crate::config::test_env::restore_var(var, orig);
    }

    #[test]
    fn resolve_prefers_project_dir_over_root() {
        let dir = tempfile::tempdir().unwrap();
        write_at(&scope_file(dir.path()), "scopes:\n  - name: Project\n");
        write_at(&dir.path().join(SCOPE_FILE), "scopes:\n  - name: Root\n");

        let resolved = resolve_scope_path(dir.path()).unwrap();
        assert_eq!(resolved, scope_file(dir.path()));
        let yaml = read_scope_yaml(dir.path()).unwrap();
        assert_eq!(yaml.scopes[0].name, "Project");
    }

    #[test]
    fn resolve_prefers_root_over_home() {
        let dir = tempfile::tempdir().unwrap();
        write_at(&dir.path().join(SCOPE_FILE), "scopes:\n  - name: Root\n");

        with_temp_home(dir.path(), || {
            let resolved = resolve_scope_path(dir.path()).unwrap();
            assert_eq!(resolved, dir.path().join(SCOPE_FILE));
        });
    }

    #[test]
    fn resolve_falls_back_to_home() {
        let dir = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        write_at(&home.path().join(SCOPE_FILE), "scopes:\n  - name: Home\n");

        with_temp_home(home.path(), || {
            let resolved = resolve_scope_path(dir.path()).unwrap();
            let yaml = read_scope_yaml(dir.path()).unwrap();
            assert_eq!(resolved, home.path().join(SCOPE_FILE));
            assert_eq!(yaml.scopes[0].name, "Home");
        });
    }

    #[test]
    fn resolve_none_when_no_file_anywhere() {
        let dir = tempfile::tempdir().unwrap();
        let empty_home = tempfile::tempdir().unwrap();

        with_temp_home(empty_home.path(), || {
            assert!(resolve_scope_path(dir.path()).is_none());
        });
    }
}
