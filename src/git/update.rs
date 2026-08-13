use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use crate::ai::{AiProvider, ProviderRegistry, ScopeRequest};
use crate::config::AiConfigLoader;
use crate::git::project::{self, ScopeYaml, TypeScopeItem};

/// 提交给 LLM 的目录列表上限（超出截断，保护提示词长度）
const MAX_DIRS_INPUT: usize = 200;
/// 通用噪音目录：即使未被 gitignore 也排除
const NOISE_DIRS: &[&str] = &["node_modules", "target", "dist", "vendor", "out"];

/// 解析仓库根目录；不在 git 仓库中则使用当前目录
fn repo_root(base: &Path) -> PathBuf {
    if let Ok(out) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(base)
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return PathBuf::from(s);
            }
        }
    }
    base.to_path_buf()
}

fn is_gitignored(base: &Path, path: &Path) -> bool {
    let rel = match path.strip_prefix(base) {
        Ok(r) => r.to_string_lossy().to_string(),
        Err(_) => path.to_string_lossy().to_string(),
    };
    let out = match Command::new("git")
        .args(["check-ignore", "-q"])
        .arg(&rel)
        .current_dir(base)
        .output()
    {
        Ok(o) => o,
        Err(_) => return false,
    };
    out.status.success()
}

fn walk(base: &Path, dir: &Path, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || NOISE_DIRS.contains(&name.as_str()) {
            continue;
        }
        if is_gitignored(base, &path) {
            continue;
        }
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        out.push(rel);
        walk(base, &path, out);
    }
}

/// 收集项目全部目录（排除隐藏目录、噪音目录与 gitignore 忽略的目录）
fn collect_dirs(base: &Path) -> Vec<String> {
    let mut dirs = Vec::new();
    walk(base, base, &mut dirs);
    dirs.sort();
    dirs.dedup();
    dirs
}

/// 从 git log 提交消息中提取历史 scope（conventional commit 格式: type(scope): msg）
fn collect_history_scopes(base: &Path) -> Vec<String> {
    let out = match Command::new("git")
        .args(["log", "--format=%s"])
        .current_dir(base)
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };
    let s = String::from_utf8_lossy(&out.stdout);
    let re = match regex::Regex::new(r"^[^(]+\(([^)]+)\):") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut seen = HashSet::new();
    let mut scopes = Vec::new();
    for line in s.lines() {
        if let Some(caps) = re.captures(line) {
            for scope in caps[1].split(',') {
                let scope = scope.trim();
                if !scope.is_empty() && seen.insert(scope.to_string()) {
                    scopes.push(scope.to_string());
                }
            }
        }
    }
    scopes
}

#[derive(Deserialize)]
struct ScopeResponse {
    #[serde(default)]
    scopes: Vec<TypeScopeItem>,
}

/// 解析 LLM 返回的 scope JSON，兼容 markdown 代码块包裹
fn parse_scopes(raw: &str) -> anyhow::Result<Vec<TypeScopeItem>> {
    let raw = raw.trim();
    let cleaned = raw
        .strip_prefix("```json")
        .or_else(|| raw.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map(|s| s.trim())
        .unwrap_or(raw);
    let resp: ScopeResponse = serde_json::from_str(cleaned)
        .map_err(|e| anyhow::anyhow!("LLM 返回无法解析: {e}\n原始内容: {raw}"))?;
    let mut seen = HashSet::new();
    Ok(resp
        .scopes
        .into_iter()
        .filter(|s| !s.name.trim().is_empty())
        .filter(|s| seen.insert(s.name.clone()))
        .collect())
}

/// 合并现有配置与 LLM 生成结果：
/// - 默认：保留现有条目，追加新增
/// - --prune：以生成结果为准，同名时保留现有 docs
fn merge_scopes(existing: ScopeYaml, generated: Vec<TypeScopeItem>, prune: bool) -> ScopeYaml {
    let existing_docs: HashMap<&str, &str> = existing
        .scopes
        .iter()
        .filter_map(|s| s.docs.as_deref().map(|d| (s.name.as_str(), d)))
        .collect();

    let mut merged: Vec<TypeScopeItem> = Vec::new();
    let mut names: HashSet<String> = HashSet::new();

    if prune {
        for mut g in generated {
            if g.docs.is_none() {
                if let Some(d) = existing_docs.get(g.name.as_str()) {
                    g.docs = Some((*d).to_string());
                }
            }
            if names.insert(g.name.clone()) {
                merged.push(g);
            }
        }
    } else {
        merged.extend(existing.scopes.clone());
        names.extend(merged.iter().map(|s| s.name.clone()));
        for g in generated {
            if names.insert(g.name.clone()) {
                merged.push(g);
            }
        }
    }

    ScopeYaml { types: existing.types, scopes: merged }
}

fn scopes_equal(a: &ScopeYaml, b: &ScopeYaml) -> bool {
    if a.scopes.len() != b.scopes.len() {
        return false;
    }
    a.scopes
        .iter()
        .zip(b.scopes.iter())
        .all(|(x, y)| x.name == y.name && x.docs == y.docs)
}

/// 核心流程（可注入 AI provider，便于测试）
pub fn run_impl(
    root: &Path,
    provider: &dyn AiProvider,
    prune: bool,
) -> anyhow::Result<()> {
    let mut dirs = collect_dirs(root);
    let truncated = dirs.len() > MAX_DIRS_INPUT;
    dirs.truncate(MAX_DIRS_INPUT);
    let history = collect_history_scopes(root);
    let existing = project::read_scope_yaml(root);

    let raw = provider.generate_scopes(&ScopeRequest {
        dirs,
        history_scopes: history,
        existing_yaml: existing.as_ref().map(project::serialize).unwrap_or_default(),
    })?;
    if truncated {
        println!("(目录数量超过 {MAX_DIRS_INPUT} 个，已截断后提交 LLM)");
    }

    let generated = parse_scopes(&raw)?;
    if generated.is_empty() {
        anyhow::bail!("LLM 未返回任何 scope");
    }

    let existing = existing.unwrap_or_else(|| ScopeYaml {
        types: project::default_types(),
        scopes: vec![],
    });
    let final_yaml = merge_scopes(existing.clone(), generated, prune);

    if scopes_equal(&existing, &final_yaml) {
        println!("scope 已是最新，无需更新");
        return Ok(());
    }

    let removed: Vec<&str> = existing
        .scopes
        .iter()
        .filter(|s| !final_yaml.scopes.iter().any(|f| f.name == s.name))
        .map(|s| s.name.as_str())
        .collect();
    let added: Vec<&str> = final_yaml
        .scopes
        .iter()
        .filter(|s| !existing.scopes.iter().any(|e| e.name == s.name))
        .map(|s| s.name.as_str())
        .collect();

    project::write_scope_yaml(root, &final_yaml)?;
    println!("已更新 .project/.git-style-scope.yaml");
    if !added.is_empty() {
        println!("新增 scope: {}", added.join(", "));
    }
    if !removed.is_empty() {
        println!("移除 scope: {}", removed.join(", "));
    }
    Ok(())
}

pub fn run(
    prune: bool,
    config_dir: Option<String>,
    api_key: Option<String>,
) -> anyhow::Result<()> {
    let base = std::env::current_dir()?;
    let root = repo_root(&base);
    let cfg = AiConfigLoader::new()
        .with_config_dir(config_dir)
        .with_api_key(api_key)
        .build()?;
    let provider_name = cfg.provider.clone();
    let provider = ProviderRegistry::new().create(&provider_name, cfg)?;
    run_impl(&root, provider.as_ref(), prune)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{Request, ScopeRequest};
    use std::sync::Arc;
    use std::sync::Mutex;

    struct FakeProvider {
        result: Arc<Mutex<String>>,
    }

    impl AiProvider for FakeProvider {
        fn generate(&self, _req: &Request) -> anyhow::Result<String> {
            Ok(String::new())
        }
        fn generate_scopes(&self, _req: &ScopeRequest) -> anyhow::Result<String> {
            Ok(self.result.lock().unwrap().clone())
        }
    }

    fn fake(result: &str) -> FakeProvider {
        FakeProvider { result: Arc::new(Mutex::new(result.to_string())) }
    }

    fn init_git_repo(dir: &Path) {
        Command::new("git").arg("init").current_dir(dir).output().unwrap();
    }

    fn git_commit(dir: &Path, message: &str) {
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args([
                "-c", "user.name=test",
                "-c", "user.email=test@example.com",
                "commit", "-m", message,
            ])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    fn write_scope_yaml(dir: &Path, content: &str) {
        let file = project::scope_file(dir);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }

    #[test]
    fn collect_dirs_excludes_hidden_noise_and_gitignored() {
        let dir = tempfile::tempdir().unwrap();
        init_git_repo(dir.path());
        for d in ["src/pipeline", "docs", "node_modules/x", "target"] {
            std::fs::create_dir_all(dir.path().join(d)).unwrap();
        }
        std::fs::create_dir_all(dir.path().join(".hidden")).unwrap();
        std::fs::write(dir.path().join(".gitignore"), "target/\n").unwrap();

        let dirs = collect_dirs(dir.path());
        assert_eq!(dirs, vec!["docs".to_string(), "src".to_string(), "src/pipeline".to_string()]);
    }

    #[test]
    fn collect_history_scopes_parses_conventional_commits() {
        let dir = tempfile::tempdir().unwrap();
        init_git_repo(dir.path());
        std::fs::write(dir.path().join("a.txt"), "a").unwrap();
        git_commit(dir.path(), "feat(core): 添加核心功能");
        std::fs::write(dir.path().join("b.txt"), "b").unwrap();
        git_commit(dir.path(), "fix(ui,docs): 修复界面与文档");
        std::fs::write(dir.path().join("c.txt"), "c").unwrap();
        git_commit(dir.path(), "chore: 杂项（无 scope）");

        let mut scopes = collect_history_scopes(dir.path());
        scopes.sort();
        assert_eq!(scopes, vec!["core".to_string(), "docs".to_string(), "ui".to_string()]);
    }

    #[test]
    fn parse_scopes_accepts_plain_and_fenced_json() {
        let plain = parse_scopes(r#"{"scopes": [{"name": "Core", "docs": "核心"}]}"#).unwrap();
        assert_eq!(plain[0].name, "Core");
        let fenced =
            parse_scopes("```json\n{\"scopes\": [{\"name\": \"Ui\", \"docs\": \"界面\"}]}\n```")
                .unwrap();
        assert_eq!(fenced[0].name, "Ui");
        assert!(parse_scopes("not json").is_err());
    }

    #[test]
    fn merge_keeps_existing_and_appends_new() {
        let existing = ScopeYaml {
            types: vec![],
            scopes: vec![TypeScopeItem { name: "Core".into(), docs: Some("旧说明".into()) }],
        };
        let generated = vec![
            TypeScopeItem { name: "Core".into(), docs: None },
            TypeScopeItem { name: "Pipeline".into(), docs: Some("管线".into()) },
        ];
        let merged = merge_scopes(existing, generated, false);
        assert_eq!(merged.scopes.len(), 2);
        assert_eq!(merged.scopes[0].docs.as_deref(), Some("旧说明"));
        assert_eq!(merged.scopes[1].name, "Pipeline");
    }

    #[test]
    fn merge_prune_replaces_and_keeps_docs_by_name() {
        let existing = ScopeYaml {
            types: vec![],
            scopes: vec![
                TypeScopeItem { name: "Core".into(), docs: Some("旧说明".into()) },
                TypeScopeItem { name: "Old".into(), docs: Some("已删除".into()) },
            ],
        };
        let generated = vec![TypeScopeItem { name: "Core".into(), docs: None }];
        let merged = merge_scopes(existing, generated, true);
        assert_eq!(merged.scopes.len(), 1);
        assert_eq!(merged.scopes[0].name, "Core");
        assert_eq!(merged.scopes[0].docs.as_deref(), Some("旧说明"));
    }

    #[test]
    fn run_impl_creates_file_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        init_git_repo(dir.path());
        std::fs::create_dir_all(dir.path().join("src")).unwrap();

        run_impl(
            dir.path(),
            &fake(r#"{"scopes": [{"name": "Src", "docs": "源码"}]}"#),
            false,
        )
        .unwrap();

        let yaml = project::read_scope_yaml(dir.path()).unwrap();
        assert_eq!(yaml.types.len(), 5, "文件缺失时应使用默认 type 模板");
        assert_eq!(yaml.scopes[0].name, "Src");
    }

    #[test]
    fn run_impl_appends_new_scopes() {
        let dir = tempfile::tempdir().unwrap();
        init_git_repo(dir.path());
        write_scope_yaml(
            dir.path(),
            "type:\n  - name: Feat\nscopes:\n  - name: Core\n    docs: 核心\n",
        );
        std::fs::create_dir_all(dir.path().join("src/pipeline")).unwrap();

        run_impl(
            dir.path(),
            &fake(r#"{"scopes": [{"name": "Pipeline", "docs": "管线"}, {"name": "Core", "docs": ""}]}"#),
            false,
        )
        .unwrap();

        let yaml = project::read_scope_yaml(dir.path()).unwrap();
        assert_eq!(yaml.scopes.len(), 2);
        assert_eq!(yaml.scopes[0].docs.as_deref(), Some("核心"));
        assert_eq!(yaml.scopes[1].name, "Pipeline");
    }

    #[test]
    fn run_impl_prune_removes_stale_scopes() {
        let dir = tempfile::tempdir().unwrap();
        init_git_repo(dir.path());
        write_scope_yaml(
            dir.path(),
            "type:\n  - name: Feat\nscopes:\n  - name: Core\n  - name: Stale\n",
        );

        run_impl(
            dir.path(),
            &fake(r#"{"scopes": [{"name": "Core", "docs": "核心"}]}"#),
            true,
        )
        .unwrap();

        let yaml = project::read_scope_yaml(dir.path()).unwrap();
        assert_eq!(yaml.scopes.len(), 1);
        assert_eq!(yaml.scopes[0].name, "Core");
    }

    #[test]
    fn run_impl_no_change_skips_write() {
        let dir = tempfile::tempdir().unwrap();
        init_git_repo(dir.path());
        write_scope_yaml(
            dir.path(),
            "type:\n  - name: Feat\nscopes:\n  - name: Core\n    docs: 核心\n",
        );

        run_impl(
            dir.path(),
            &fake(r#"{"scopes": [{"name": "Core", "docs": "核心"}]}"#),
            false,
        )
        .unwrap();

        let yaml = project::read_scope_yaml(dir.path()).unwrap();
        assert_eq!(yaml.scopes[0].name, "Core");
    }

    #[test]
    fn run_impl_empty_llm_result_errors() {
        let dir = tempfile::tempdir().unwrap();
        init_git_repo(dir.path());

        let err = run_impl(dir.path(), &fake(r#"{"scopes": []}"#), false).unwrap_err();
        assert!(err.to_string().contains("未返回任何 scope"));
    }
}
