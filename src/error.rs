use std::fmt;

#[derive(Debug)]
pub enum AgcError {
    Git(String),
    Api(String),
    Config(String),
    Parse(String),
    UserCancel,
}

impl fmt::Display for AgcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgcError::Git(msg) => write!(f, "Git 操作失败: {msg}"),
            AgcError::Api(msg) => write!(f, "API 错误: {msg}"),
            AgcError::Config(msg) => write!(f, "配置错误: {msg}"),
            AgcError::Parse(msg) => write!(f, "解析失败: {msg}"),
            AgcError::UserCancel => write!(f, "用户取消"),
        }
    }
}

impl std::error::Error for AgcError {}
