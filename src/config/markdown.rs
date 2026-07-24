use std::fs;

use crate::strutil;

use super::Config;

pub struct MarkdownLoader {
    path: String,
}

impl MarkdownLoader {
    pub fn new(path: &str) -> Self {
        Self { path: path.into() }
    }

    pub fn load(&self) -> Option<Config> {
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
