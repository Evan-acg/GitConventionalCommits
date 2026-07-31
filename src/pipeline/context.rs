use crate::commit::Entry;

/// 管线各阶段共享的上下文数据
pub struct PipelineContext {
    // ── 控制参数 ──
    pub auto_push: Option<String>,
    pub auto_commit: bool,

    // ── 第1阶段产出：DiffCollector ──
    pub diff: String,
    pub changed_files: Vec<String>,
    pub status_short: String,
    pub diff_stat: String,
    pub diff_cached_stat: String,
    pub untracked_files: Vec<String>,
    pub has_changes: bool,

    // ── 第1阶段衍生：git_info ──
    pub git_info: String,

    // ── 第2阶段产出：ContextEnricher ──
    pub extra_context: String,

    // ── 第3阶段产出：AiGenerator ──
    pub raw_response: String,

    // ── 第4阶段产出：EntryParser ──
    pub entries: Vec<Entry>,
    pub reason: String,
}

impl PipelineContext {
    pub fn new(auto_push: Option<String>, auto_commit: bool) -> Self {
        Self {
            auto_push,
            auto_commit,
            diff: String::new(),
            changed_files: Vec::new(),
            status_short: String::new(),
            diff_stat: String::new(),
            diff_cached_stat: String::new(),
            untracked_files: Vec::new(),
            has_changes: false,
            git_info: String::new(),
            extra_context: String::new(),
            raw_response: String::new(),
            entries: Vec::new(),
            reason: String::new(),
        }
    }

    /// DiffCollector 收集完原始数据后调用此方法构建 git_info 字符串
    pub fn build_git_info(&mut self) {
        let mut info = String::new();
        if !self.status_short.is_empty() {
            info.push_str(&format!("--- 工作区状态 ---\n{}\n", self.status_short));
        }
        if !self.diff_cached_stat.is_empty() {
            info.push_str(&format!("--- 已暂存变更摘要 ---\n{}\n", self.diff_cached_stat));
        }
        if !self.diff_stat.is_empty() {
            info.push_str(&format!("--- 未暂存变更摘要 ---\n{}\n", self.diff_stat));
        }
        if !self.untracked_files.is_empty() {
            // untracked_content 由外部注入，这里留空，后续可扩展
        }
        self.git_info = info;
    }
}
