pub mod fd;
pub mod rg;

pub use fd::*;
pub use rg::*;

pub trait SearchBackend {
    fn rg_context(&self, files: &[String], pattern: &str) -> String;
    fn fd_context(&self, files: &[String], pattern: &str) -> String;
}

pub struct RealSearch;

impl SearchBackend for RealSearch {
    fn rg_context(&self, files: &[String], pattern: &str) -> String {
        rg::rg_context(files, pattern)
    }
    fn fd_context(&self, files: &[String], pattern: &str) -> String {
        fd::fd_context(files, pattern)
    }
}

pub fn merge_context(rg: &str, fd: &str) -> String {
    let mut result = String::new();
    if !rg.is_empty() {
        result.push_str("---\n变更文件结构上下文:\n");
        result.push_str(rg);
    }
    if !fd.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str("---\n关联文件上下文:\n");
        result.push_str(fd);
    }
    result
}
