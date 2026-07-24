use crate::commit::{AiResponse, Entry};

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
    println!("\n{}", crate::ui::color::cyan("生成的提交消息:"));
    println!("{}", crate::ui::color::bold(&msg));
    if !entry.files.is_empty() {
        println!("\n{}", crate::ui::color::cyan("关联文件:"));
        for f in &entry.files {
            println!("  {f}");
        }
    }
    print!("\n{}", crate::ui::color::cyan("确认提交？回复 ok 执行: "));
    std::io::Write::flush(&mut std::io::stdout()).ok();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    input.trim() == "ok"
}

/// 从 AI 返回的文本中提取 JSON 并解析为 AiResponse。
///
/// 优先使用 serde_json 直接解析。如果失败，会尝试去除 markdown 代码块标记后再解析
///（作为对不支持 response_format 参数的低版本模型的兜底兼容）。
pub fn parse_entries(raw: &str) -> (Vec<Entry>, String) {
    let raw = raw.trim();
    if raw.is_empty() {
        return (vec![], String::new());
    }

    // 直接尝试 JSON 解析（response_format 启用后大多数模型会返回纯 JSON）
    if let Ok(resp) = serde_json::from_str::<AiResponse>(raw) {
        let total = resp.data.len();
        let (valid, reason) = resp.into_valid();
        let skipped = total - valid.len();
        if skipped > 0 {
            eprintln!(
                "{}",
                crate::ui::color::yellow(&format!("警告: AI 返回的 {skipped} 个 entry 缺少必要字段（type/scope/message），已跳过"))
            );
        }
        return (valid, reason);
    }

    // 兜底：去除 markdown 代码块标记后重试（兼容不支持 response_format 的模型）
    let cleaned = raw
        .strip_prefix("```json")
        .or_else(|| raw.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .map(|s| s.trim())
        .unwrap_or(raw);

    match serde_json::from_str::<AiResponse>(cleaned) {
        Ok(resp) => {
            let total = resp.data.len();
            let (valid, reason) = resp.into_valid();
            let skipped = total - valid.len();
            if skipped > 0 {
                eprintln!(
                    "{}",
                    crate::ui::color::yellow(&format!("警告: AI 返回的 {skipped} 个 entry 缺少必要字段（type/scope/message），已跳过"))
                );
            }
            (valid, reason)
        }
        Err(e) => {
            eprintln!(
                "{}",
                crate::ui::color::yellow(&format!("警告: AI 返回内容无法解析为合法 JSON，解析错误: {e}"))
            );
            (vec![], String::new())
        }
    }
}
