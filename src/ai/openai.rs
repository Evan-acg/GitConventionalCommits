use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use ureq::config::Config;
use ureq::tls::{TlsConfig, TlsProvider};

use super::{AiProvider, Request};

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ApiError {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    error: Option<ApiError>,
}

pub struct OpenAI {
    api_key: String,
    model: String,
    base_url: String,
    agent: ureq::Agent,
}

impl OpenAI {
    pub fn new(api_key: String, model: String, base_url: String) -> Self {
        Self {
            api_key,
            model,
            base_url,
            agent: Config::builder()
                .timeout_global(Some(Duration::from_secs(120)))
                .tls_config(
                    TlsConfig::builder()
                        .provider(TlsProvider::NativeTls)
                        .build(),
                )
                .build()
                .into(),
        }
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = std::env::var("MESSAGE_API_KEY")
            .map_err(|_| anyhow::anyhow!("MESSAGE_API_KEY 未设置"))?;
        let model = std::env::var("OPENAI_MODEL")
            .unwrap_or_else(|_| "deepseek-v4-flash".to_string());
        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.deepseek.com".to_string());
        Ok(Self::new(api_key, model, base_url))
    }
}

impl AiProvider for OpenAI {
    fn generate(&self, req: &Request) -> anyhow::Result<String> {
        let type_list: String = req
            .types
            .iter()
            .map(|t| format!("- {t}\n"))
            .collect();
        let scope_list: String = req
            .scopes
            .iter()
            .map(|s| format!("- {s}\n"))
            .collect();

        let system_prompt = format!(
            r#"你是一个 git commit 消息生成助手。根据以下 git 信息和可用的 type/scope 分类，生成 conventional commit 消息。

可用的 Type:
{type_list}

可用的 Scope:
{scope_list}

输出格式: 始终返回纯 JSON 对象（不要 markdown 代码块，不要额外说明），包含以下字段:
- reason: 分析说明 (中文，说明为何选择单条或多条提交)
- data: commit 消息数组，每个元素包含以下字段:
  - type: 变更类型 (必填，从可用 Type 中选择)
  - scope: 变更范围 (必填，从可用 Scope 中选择)
  - message: 中文描述 (一句话概括变更内容)
  - files: 该 commit 涉及的文件路径数组 (需要 git add 的文件)
  - detail: 变更的详细描述 (markdown 列表格式，以 - 开头列出每个具体变更)

规则:
- 根据 diff 内容选择最匹配的 Type 和 Scope
- type 和 scope 为必填字段，不得为空
- 消息用中文描述变更内容
- 分析 diff 内容判断是否需要分多条 commit
- 如果 diff 包含多个独立不相关的变更，为每组独立变更输出一条 commit
- 如果所有变更是相关的、完成单一目标，只输出一条
- 只返回 JSON 对象，不要 markdown 代码块、不要额外说明

示例输出（这是唯一合法格式）:
{{"reason": "本次变更中的修改紧密相关，适合作为单条提交", "data": [{{"type": "Feat", "scope": "Git", "message": "添加新的 git 函数", "files": ["internal/git/git.go"], "detail": "- 新增 StatusShort 函数\n- 新增 DiffStat 函数"}}]}}"#
        );

        let mut user_content = "请根据以下 git 信息生成 commit 消息:\n\n".to_string();
        if !req.git_info.is_empty() {
            user_content.push_str(&format!("--- 工作区状态 ---\n{}\n\n", req.git_info));
        }
        user_content.push_str(&format!("--- 完整 diff ---\n{}", req.diff));
        if !req.extra_context.is_empty() {
            user_content.push_str(&format!("\n\n--- 变更上下文 ---\n{}", req.extra_context));
        }

        let chat_req = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_content,
                },
            ],
            response_format: Some(json!({"type": "json_object"})),
        };

        let url = format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'));
        let body = serde_json::to_string(&chat_req)?;

        let mut resp = self
            .agent
            .post(&url)
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .content_type("application/json")
            .send(&body)?;

        let body_text = resp.body_mut().read_to_string()?;
        let chat_resp: ChatResponse = serde_json::from_str(&body_text)?;

        if chat_resp.choices.is_empty() {
            if let Some(err) = &chat_resp.error {
                anyhow::bail!("API 返回错误: {} ({})", err.message, err.error_type);
            }
            anyhow::bail!("API 返回空结果（无错误信息）");
        }

        Ok(chat_resp.choices[0].message.content.trim().to_string())
    }
}
