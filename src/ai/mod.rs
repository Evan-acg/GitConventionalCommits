pub mod openai;

#[derive(Debug, Clone)]
pub struct Request {
    pub types: Vec<String>,
    pub scopes: Vec<String>,
    pub diff: String,
    pub git_info: String,
    pub extra_context: String,
}

pub trait AiProvider: Send + Sync {
    fn generate(&self, req: &Request) -> anyhow::Result<String>;
}
