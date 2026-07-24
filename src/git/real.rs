use std::process::Command;

use super::GitBackend;

pub struct RealGit;

impl GitBackend for RealGit {
    fn diff(&self) -> anyhow::Result<String> {
        let out = Command::new("git").args(["diff"]).output()?;
        if !out.status.success() {
            anyhow::bail!("git diff 失败");
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    fn changed_files(&self) -> anyhow::Result<Vec<String>> {
        let out = Command::new("git").args(["diff", "--name-only"]).output()?;
        if !out.status.success() {
            anyhow::bail!("git diff --name-only 失败");
        }
        let s = String::from_utf8_lossy(&out.stdout);
        Ok(s.split_whitespace().map(|s| s.to_string()).collect())
    }

    fn status_short(&self) -> anyhow::Result<String> {
        let out = Command::new("git").args(["status", "--short"]).output()?;
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    fn diff_stat(&self) -> anyhow::Result<String> {
        let out = Command::new("git").args(["diff", "--stat"]).output()?;
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    fn diff_cached_stat(&self) -> anyhow::Result<String> {
        let out = Command::new("git").args(["diff", "--cached", "--stat"]).output()?;
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    fn ls_untracked(&self) -> anyhow::Result<Vec<String>> {
        let out = Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .output()?;
        let s = String::from_utf8_lossy(&out.stdout);
        Ok(s.split_whitespace().map(|s| s.to_string()).collect())
    }

    fn untracked_content(&self, paths: &[String]) -> String {
        let mut result = String::new();
        for path in paths {
            let meta = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.len() > 100 * 1024 {
                result.push_str(&format!("--- 未跟踪文件 (超过100KB，已跳过): {path} ---\n"));
                continue;
            }
            let data = match std::fs::read(path) {
                Ok(d) => d,
                Err(_) => continue,
            };
            if data.contains(&0u8) {
                result.push_str(&format!("--- 未跟踪文件 (二进制): {path} ---\n"));
                continue;
            }
            let content = String::from_utf8_lossy(&data);
            result.push_str(&format!("--- 未跟踪文件: {path} ---\n{content}\n"));
        }
        result
    }

    fn stage_all(&self) -> anyhow::Result<()> {
        crate::commit::executor::stage_all()
    }

    fn stage_files(&self, files: &[String]) -> anyhow::Result<()> {
        crate::commit::executor::stage_files(files)
    }

    fn commit(&self, msg: &str) -> anyhow::Result<()> {
        crate::commit::executor::commit(msg)
    }
}
