#[derive(Debug, Clone)]
pub struct Request {
    pub types: Vec<String>,
    pub scopes: Vec<String>,
    pub diff: String,
    pub git_info: String,
    pub extra_context: String,
}

pub trait AiProvider {
    fn generate(
        &self,
        ctx: &crate::app::WorkflowContext,
        req: &Request,
    ) -> anyhow::Result<String>;
}