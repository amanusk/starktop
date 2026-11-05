use thiserror::Error;

#[derive(Error, Debug)]
pub enum StarktopError {
    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("ABI decode error: {0}")]
    Decode(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for StarktopError {
    fn from(e: serde_json::Error) -> Self {
        StarktopError::Serialization(e.to_string())
    }
}

impl From<reqwest::Error> for StarktopError {
    fn from(e: reqwest::Error) -> Self {
        StarktopError::Rpc(e.to_string())
    }
}

impl From<toml::de::Error> for StarktopError {
    fn from(e: toml::de::Error) -> Self {
        StarktopError::Config(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, StarktopError>;
