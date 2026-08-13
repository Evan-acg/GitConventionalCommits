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
    #[serde(default)]
    pub docs: Option<String>,
}

pub fn scope_file(base: &Path) -> PathBuf {
    base.join(PROJECT_DIR).join(SCOPE_FILE)
}

pub fn read_scope_yaml(base: &Path) -> Option<ScopeYaml> {
    let data = fs::read_to_string(scope_file(base)).ok()?;
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
}
