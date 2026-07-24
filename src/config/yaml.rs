use std::fs;

use serde::Deserialize;

use super::Config;

#[derive(Deserialize)]
struct TypeScopeItem {
    name: String,
}

#[derive(Deserialize)]
struct LazyGitConfig {
    #[serde(rename = "type")]
    types: Option<Vec<TypeScopeItem>>,
    scopes: Option<Vec<TypeScopeItem>>,
}

pub struct YamlLoader {
    path: String,
}

impl YamlLoader {
    pub fn new(path: &str) -> Self {
        Self {
            path: if path.is_empty() { ".lazygit.yaml".into() } else { path.into() },
        }
    }

    pub fn load(&self) -> Option<Config> {
        let data = fs::read_to_string(&self.path).ok()?;
        let cfg = serde_yml::from_str::<LazyGitConfig>(&data).ok()?;
        let types: Vec<String> = cfg
            .types
            .unwrap_or_default()
            .iter()
            .map(|t| t.name.clone())
            .collect();
        let scopes: Vec<String> = cfg
            .scopes
            .unwrap_or_default()
            .iter()
            .map(|s| s.name.clone())
            .collect();
        if types.is_empty() {
            return None;
        }
        Some(Config { types, scopes })
    }
}
