use serde::{Deserialize, Deserializer, Serialize};

pub mod executor;
pub mod service;

#[derive(Deserialize)]
#[serde(untagged)]
enum Detail {
    Text(String),
    Lines(Vec<String>),
}

fn deserialize_detail<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    match Detail::deserialize(deserializer)? {
        Detail::Text(detail) => Ok(detail),
        Detail::Lines(lines) => Ok(lines
            .into_iter()
            .map(|line| format!("- {line}"))
            .collect::<Vec<_>>()
            .join("\n")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub scope: String,
    pub message: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_detail")]
    pub detail: String,
}

impl Entry {
    /// 校验必填字段是否完整
    pub fn is_valid(&self) -> bool {
        !self.entry_type.is_empty() && !self.scope.is_empty() && !self.message.is_empty()
    }
}

#[derive(Debug, Deserialize)]
pub struct AiResponse {
    pub reason: String,
    pub data: Vec<Entry>,
}

impl AiResponse {
    /// 解析后过滤掉无效 entry，并返回有效数据
    pub fn into_valid(self) -> (Vec<Entry>, String) {
        let valid: Vec<Entry> = self.data.into_iter().filter(|e| e.is_valid()).collect();
        (valid, self.reason)
    }
}
