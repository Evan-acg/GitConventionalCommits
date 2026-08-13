use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::json;
use ureq::config::Config;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

use super::prompt::PromptBuilder;
use super::{AiProvider, Request};
use crate::config::AiConfig;

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    reasning_effort: String,
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

/// OpenAI 兼容协议（Chat Completions）provider，只负责传输。
/// base_url 为完整 chat/completions 端点地址。
pub struct OpenAI {
    api_key: String,
    model: String,
    base_url: String,
    agent: ureq::Agent,
}

impl OpenAI {
    pub fn new(config: AiConfig) -> Self {
        Self {
            api_key: config.api_key.unwrap_or_default(),
            model: config.model,
            base_url: config.base_url,
            agent: Config::builder()
                .timeout_global(Some(Duration::from_secs(120)))
                .tls_config(
                    TlsConfig::builder()
                        .provider(TlsProvider::NativeTls)
                        // ureq 默认使用 Mozilla webpki 根且禁用系统根，
                        // SChannel 下会报 "unable to find any user-specified roots"，
                        // 显式使用平台系统根证书验证
                        .root_certs(RootCerts::PlatformVerifier)
                        .build(),
                )
                .build()
                .into(),
        }
    }

    /// 供 ProviderRegistry 注册的工厂函数
    pub fn create(config: AiConfig) -> anyhow::Result<Arc<dyn AiProvider>> {
        if config.api_key.as_deref().unwrap_or("").is_empty() {
            anyhow::bail!("openai provider 缺少 api_key，请在配置文件或 MESSAGE_API_KEY 环境变量中设置");
        }
        Ok(Arc::new(Self::new(config)))
    }
}

impl AiProvider for OpenAI {
    fn generate(&self, req: &Request) -> anyhow::Result<String> {
        self.chat(vec![
            ChatMessage {
                role: "system".to_string(),
                content: PromptBuilder::build_system(&req.types, &req.scopes),
            },
            ChatMessage {
                role: "user".to_string(),
                content: PromptBuilder::build_user(req),
            },
        ])
    }

    fn generate_scopes(&self, req: &super::ScopeRequest) -> anyhow::Result<String> {
        self.chat(vec![
            ChatMessage {
                role: "system".to_string(),
                content: PromptBuilder::build_scope_system(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: PromptBuilder::build_scope_user(req),
            },
        ])
    }
}

impl OpenAI {
    /// 发送 chat/completions 请求并返回首个 choice 的文本内容
    fn chat(&self, messages: Vec<ChatMessage>) -> anyhow::Result<String> {
        let chat_req = ChatRequest {
            model: self.model.clone(),
            messages,
            reasning_effort: "low".to_string(),
            response_format: Some(json!({"type": "json_object"})),
        };

        let url = self.base_url.trim_end_matches('/').to_string();
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
