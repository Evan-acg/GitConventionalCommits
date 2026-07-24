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
