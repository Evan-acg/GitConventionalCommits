use std::fs;
use std::path::Path;
use std::process::Command;

use super::project;

const SCOPE_YAML_TEMPLATE: &str = r#"# 项目 type/scope 定义（与 .lazygit.yaml 同格式）
type:
  - name: Feat
    docs: 新特性
  - name: Fix
    docs: 修复缺陷
  - name: Docs
    docs: 文档变更
  - name: Refactor
    docs: 重构代码
  - name: Test
    docs: 测试相关
scopes:
  - name: Core
    docs: 核心逻辑
  - name: UI
    docs: 界面相关
  - name: Build
    docs: 构建脚本
"#;

fn is_git_repo(base: &Path) -> bool {
    let out = match Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(base)
        .output()
    {
        Ok(out) => out,
        Err(_) => return false,
    };
    out.status.success()
        && String::from_utf8_lossy(&out.stdout).trim() == "true"
}

fn git_init(base: &Path) -> anyhow::Result<()> {
    if is_git_repo(base) {
        println!("当前目录已在 git 仓库中，跳过 git init");
        return Ok(());
    }
    let status = Command::new("git").arg("init").current_dir(base).status()?;
    if !status.success() {
        anyhow::bail!("git init 失败");
    }
    println!("已初始化 git 仓库");
    Ok(())
}

fn scaffold_project(base: &Path, force: bool) -> anyhow::Result<()> {
    let file = project::scope_file(base);
    if file.exists() && !force {
        println!("{} 已存在，如需覆盖请添加 --force", file.display());
        return Ok(());
    }
    fs::create_dir_all(file.parent().unwrap()).map_err(|e| {
        anyhow::anyhow!("无法创建目录 {}: {e}", file.parent().unwrap().display())
    })?;
    fs::write(&file, SCOPE_YAML_TEMPLATE)?;
    println!("已生成配置文件: {}", file.display());
    Ok(())
}

pub fn run(force: bool) -> anyhow::Result<()> {
    let base = std::env::current_dir()?;
    git_init(&base)?;
    scaffold_project(&base, force)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_init_creates_repo_in_plain_dir() {
        let dir = tempfile::tempdir().unwrap();
        git_init(dir.path()).unwrap();
        assert!(dir.path().join(".git").exists());
    }

    #[test]
    fn git_init_skips_existing_repo() {
        let dir = tempfile::tempdir().unwrap();
        git_init(dir.path()).unwrap();
        git_init(dir.path()).unwrap();
        assert!(dir.path().join(".git").exists());
    }

    #[test]
    fn scaffold_creates_project_scope_file() {
        let dir = tempfile::tempdir().unwrap();
        scaffold_project(dir.path(), false).unwrap();
        let file = dir.path().join(".project").join(".git-style-scope.yaml");
        assert!(file.exists());
        let content = fs::read_to_string(file).unwrap();
        assert!(content.contains("type:"));
        assert!(content.contains("scopes:"));
    }

    #[test]
    fn scaffold_skips_existing_without_force() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(".project").join(".git-style-scope.yaml");
        fs::create_dir_all(dir.path().join(".project")).unwrap();
        fs::write(&file, "existing").unwrap();

        scaffold_project(dir.path(), false).unwrap();
        assert_eq!(fs::read_to_string(file).unwrap(), "existing");
    }

    #[test]
    fn scaffold_overwrites_with_force() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(".project").join(".git-style-scope.yaml");
        fs::create_dir_all(dir.path().join(".project")).unwrap();
        fs::write(&file, "existing").unwrap();

        scaffold_project(dir.path(), true).unwrap();
        let content = fs::read_to_string(file).unwrap();
        assert!(content.contains("type:"));
    }
}
