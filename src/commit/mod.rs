use serde::{Deserialize, Serialize};

pub mod executor;
pub mod service;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub scope: String,
    pub message: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub detail: String,
}

#[derive(Debug, Deserialize)]
pub struct AiResponse {
    pub reason: String,
    pub data: Vec<Entry>,
}
