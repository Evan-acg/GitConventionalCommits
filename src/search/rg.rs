use std::process::Command;

const DEFAULT_PATTERN: &str =
    "^(func|function|def|class|type|struct|interface|impl|fn|pub|export|const|let|var|module|trait|enum)\\b";

pub fn rg_context(files: &[String], pattern: &str) -> String {
    if which("rg").is_err() {
        eprintln!("建议安装 ripgrep (rg) 以获得更精准的 commit 消息");
        return String::new();
    }
    if files.is_empty() {
        return String::new();
    }
    let p = if pattern.is_empty() {
        DEFAULT_PATTERN
    } else {
        pattern
    };

    let mut args = vec!["--line-number".to_string(), "-e".to_string(), p.to_string()];
    for f in files {
        args.push(f.clone());
    }

    let out = Command::new("rg").args(&args).output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => String::new(),
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
