use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

const MAX_CONTEXT_LINES: usize = 500;

pub fn fd_context(changed_files: &[String], pattern: &str) -> String {
    if which("fd").is_err() {
        eprintln!("建议安装 fd (fd-find) 以获得更精准的 commit 消息");
        return String::new();
    }
    if changed_files.is_empty() {
        return String::new();
    }

    let mut sections: Vec<String> = Vec::new();

    if !pattern.is_empty() {
        if let Ok(out) = Command::new("fd").arg(pattern).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            if !s.trim().is_empty() {
                sections.push(format!("自定义搜索结果:\n{}", s.trim()));
            }
        }
    }

    let mut seen_stems: HashSet<String> = HashSet::new();
    for f in changed_files {
        let path = Path::new(f);
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if !seen_stems.insert(stem.clone()) {
            continue;
        }

        let glob = format!("**/{stem}.*");
        if let Ok(out) = Command::new("fd")
            .args(["--type", "f", "--glob", &glob])
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            let related: Vec<&str> = s.lines().filter(|line| line.trim() != f).collect();
            if !related.is_empty() {
                sections.push(format!("{f} 的关联文件:\n{}", related.join("\n")));
            }
        }
    }

    let mut seen_dirs = HashSet::new();
    for f in changed_files {
        let dir = Path::new(f).parent().unwrap_or(Path::new("."));
        let dir_str = dir.to_string_lossy().to_string();
        if !seen_dirs.insert(dir_str.clone()) {
            continue;
        }

        if let Ok(out) = Command::new("fd")
            .args(["--type", "f", "--max-depth", "1", ".", &dir_str])
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            let dir_files: Vec<String> = s.lines().map(|l| format!("  {l}")).collect();
            if !dir_files.is_empty() {
                sections.push(format!("{dir_str}/ 目录文件:\n{}", dir_files.join("\n")));
            }
        }
    }

    let result = sections.join("\n");
    let lines: Vec<&str> = result.lines().collect();
    if lines.len() > MAX_CONTEXT_LINES {
        let trimmed: Vec<&str> = lines[..MAX_CONTEXT_LINES].to_vec();
        format!("{}\n... (已截断)", trimmed.join("\n"))
    } else {
        result
    }
}

fn which(cmd: &str) -> anyhow::Result<()> {
    let status = if cfg!(windows) {
        Command::new("where")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .status()
    } else {
        Command::new("which")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .status()
    };
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => anyhow::bail!("not found"),
    }
}
