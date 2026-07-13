use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub scope: String,
    pub message: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub detail: String,
}

#[derive(Debug, Deserialize)]
pub struct AiResponse {
    pub reason: String,
    pub data: Vec<Entry>,
}

pub fn format_message(entry: &Entry) -> String {
    let msg = format!("{}({}): {}", entry.entry_type, entry.scope, entry.message);
    if entry.detail.is_empty() {
        msg
    } else {
        format!("{msg}\n\n{}", entry.detail)
    }
}

pub fn confirm_entry(entry: &Entry) -> bool {
    let msg = format_message(entry);
    println!("\n生成的提交消息:");
    println!("{msg}");
    if !entry.files.is_empty() {
        println!("\n关联文件:");
        for f in &entry.files {
            println!("  {f}");
        }
    }
    print!("\n确认提交？回复 ok 执行: ");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    input.trim() == "ok"
}

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

pub fn parse_entries(raw: &str) -> (Vec<Entry>, String) {
    let raw = raw.trim();
    if raw.is_empty() {
        return (vec![], String::new());
    }
    match serde_json::from_str::<AiResponse>(raw) {
        Ok(resp) => (resp.data, resp.reason),
        Err(_) => (vec![], String::new()),
    }
}
