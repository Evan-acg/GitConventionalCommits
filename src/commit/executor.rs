use std::process::Command;

pub fn stage_all() -> anyhow::Result<()> {
    let status = Command::new("git").args(["add", "-A"]).status()?;
    if !status.success() {
        anyhow::bail!("git add -A 失败");
    }
    Ok(())
}

pub fn stage_files(files: &[String]) -> anyhow::Result<()> {
    let mut args = vec!["add".to_string()];
    args.extend_from_slice(files);
    let status = Command::new("git").args(&args).status()?;
    if !status.success() {
        anyhow::bail!("git add 失败");
    }
    Ok(())
}

pub fn commit(msg: &str) -> anyhow::Result<()> {
    let parts: Vec<&str> = msg.splitn(2, "\n\n").collect();
    let mut args = vec!["commit", "-m", parts[0]];
    if parts.len() > 1 && !parts[1].is_empty() {
        args.push("-m");
        args.push(parts[1]);
    }
    let status = Command::new("git").args(&args).status()?;
    if !status.success() {
        anyhow::bail!("git commit 失败");
    }
    Ok(())
}
