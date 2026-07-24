pub mod real;

pub trait GitBackend {
    fn diff(&self) -> anyhow::Result<String>;
    fn changed_files(&self) -> anyhow::Result<Vec<String>>;
    fn status_short(&self) -> anyhow::Result<String>;
    fn diff_stat(&self) -> anyhow::Result<String>;
    fn diff_cached_stat(&self) -> anyhow::Result<String>;
    fn ls_untracked(&self) -> anyhow::Result<Vec<String>>;
    fn untracked_content(&self, paths: &[String]) -> String;
    fn stage_all(&self) -> anyhow::Result<()>;
    fn stage_files(&self, files: &[String]) -> anyhow::Result<()>;
    fn commit(&self, msg: &str) -> anyhow::Result<()>;
}
