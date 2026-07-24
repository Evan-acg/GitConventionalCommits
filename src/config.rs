use crate::strutil;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
#[allow(dead_code)]
struct TypeScopeItem {
    name: String,
    docs: Option<String>,
}

#[derive(Deserialize)]
struct LazyGitConfig {
    #[serde(rename = "type")]
    types: Option<Vec<TypeScopeItem>>,
    scopes: Option<Vec<TypeScopeItem>>,
}

pub fn load(skill_path: &str, lazygit_path: &str) -> (Vec<String>, Vec<String>) {
    let lp = if lazygit_path.is_empty() {
        ".lazygit.yaml"
    } else {
        lazygit_path
    };

    if let Ok(data) = fs::read_to_string(lp) {
        if let Ok(cfg) = serde_yml::from_str::<LazyGitConfig>(&data) {
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
            if !types.is_empty() {
                return (types, scopes);
            }
        }
    }

    if !skill_path.is_empty() {
        if let Ok(data) = fs::read_to_string(skill_path) {
            let re = regex::Regex::new(r"\|\s*`([^`]+)`\s*\|").unwrap();
            let types: Vec<String> = re
                .captures_iter(&data)
                .map(|c| strutil::capitalize(c[1].trim()))
                .filter(|s| !s.is_empty())
                .collect();
            if !types.is_empty() {
                return (types, vec![]);
            }
        }
    }

    let default_types = vec![
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
    ];
    (default_types, vec![])
}
