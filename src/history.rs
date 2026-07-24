use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// 目录历史记录，按 LRU（最近使用）排序
pub struct History {
    entries: Vec<String>,
    file_path: PathBuf,
}

impl History {
    /// 加载历史文件
    ///
    /// 优先使用 XDG 规范路径 `~/.local/share/agc/history`，
    /// 回退到 `~/.agc_history`。
    pub fn load() -> Self {
        let file_path = Self::resolve_path();
        let entries = if file_path.exists() {
            match fs::File::open(&file_path) {
                Ok(file) => BufReader::new(file)
                    .lines()
                    .filter_map(|line| line.ok())
                    .filter(|line| !line.is_empty())
                    .collect(),
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        };
        Self { entries, file_path }
    }

    /// 添加目录：已存在则移到最前，不存在则插入最前
    pub fn add(&mut self, path: &Path) {
        let path_str = path
            .canonicalize()
            .as_deref()
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // 已存在 → 移到最前
        if let Some(pos) = self.entries.iter().position(|e| e == &path_str) {
            let entry = self.entries.remove(pos);
            self.entries.insert(0, entry);
            return;
        }

        // 不存在 → 插入最前
        self.entries.insert(0, path_str);
    }

    /// 子串模糊匹配（大小写不敏感）
    pub fn filter(&self, pattern: &str) -> Vec<String> {
        let lower = pattern.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.to_lowercase().contains(&lower))
            .cloned()
            .collect()
    }

    /// 持久化到文件
    pub fn save(&self) {
        if let Some(parent) = self.file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut file) = fs::File::create(&self.file_path) {
            for entry in &self.entries {
                let _ = writeln!(file, "{entry}");
            }
        }
    }

    /// 调用外部 fzf 让用户从选项中选择，回退到序号输入
    pub fn select_fzf(options: &[String]) -> Option<String> {
        if options.is_empty() {
            return None;
        }
        if options.len() == 1 {
            return Some(options[0].clone());
        }

        // 尝试 fzf
        if let Some(selected) = Self::try_fzf(options) {
            return Some(selected);
        }

        // 回退到序号输入
        Self::fallback_select(options)
    }

    fn try_fzf(options: &[String]) -> Option<String> {
        use std::process::{Command, Stdio};

        let mut child = Command::new("fzf")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        {
            let stdin = child.stdin.as_mut()?;
            for opt in options {
                let _ = writeln!(stdin, "{opt}");
            }
        }

        let output = child.wait().ok()?;
        if !output.success() {
            return None; // 用户取消
        }

        let stdout = child.stdout.take()?;
        let result = std::io::read_to_string(stdout).ok()?;
        let result = result.trim().to_string();
        if result.is_empty() {
            return None;
        }
        Some(result)
    }

    fn fallback_select(options: &[String]) -> Option<String> {
        println!("\n匹配到多个目录，请选择：");
        for (i, opt) in options.iter().enumerate() {
            println!("  {}. {}", i + 1, opt);
        }
        print!("输入序号 (1-{}): ", options.len());
        std::io::Write::flush(&mut std::io::stdout()).ok()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok()?;
        let index: usize = input.trim().parse().ok()?;
        if index >= 1 && index <= options.len() {
            Some(options[index - 1].clone())
        } else {
            None
        }
    }

    fn resolve_path() -> PathBuf {
        // XDG: ~/.local/share/agc/history
        if let Some(data_dir) = dirs_data_dir() {
            let path = data_dir.join("agc").join("history");
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            return path;
        }
        // 回退: ~/.agc_history
        dirs_home_dir()
            .map(|h| h.join(".agc_history"))
            .unwrap_or_else(|| PathBuf::from(".agc_history"))
    }
}

#[cfg(target_family = "unix")]
fn dirs_data_dir() -> Option<PathBuf> {
    std::env::var("XDG_DATA_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".local").join("share"))
        })
}

#[cfg(not(target_family = "unix"))]
fn dirs_data_dir() -> Option<PathBuf> {
    None
}

#[cfg(target_family = "unix")]
fn dirs_home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

#[cfg(not(target_family = "unix"))]
fn dirs_home_dir() -> Option<PathBuf> {
    None
}
