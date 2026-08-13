use std::fs;

use crate::strutil;

use super::super::source::ConfigSource;
use super::super::Config;

/// 从 SKILL.md 的 markdown 表格加载 type 配置
pub struct MarkdownSource {
    path: String,
}

impl MarkdownSource {
    pub fn new(path: &str) -> Self {
        Self { path: path.into() }
    }
}

impl ConfigSource<Config> for MarkdownSource {
    fn load(&self) -> Option<Config> {
        if self.path.is_empty() {
            return None;
        }
        let data = fs::read_to_string(&self.path).ok()?;
        let re = regex::Regex::new(r"\|\s*`([^`]+)`\s*\|").ok()?;
        let types: Vec<String> = re
            .captures_iter(&data)
            .map(|c| strutil::capitalize(c[1].trim()))
            .filter(|s| !s.is_empty())
            .collect();
        if types.is_empty() {
            return None;
        }
        Some(Config { types, scopes: vec![] })
    }
}
