use std::str::Utf8Error;

use serr::SerializeError;

#[derive(thiserror::Error, Debug, SerializeError)]
pub enum RequestError {
    #[error("Failed to acquire semaphore")]
    AcquireError(#[from] tokio::sync::AcquireError),

    #[error("Failed to send request: {0}")]
    RequestSendError(#[from] reqwest::Error),

    #[error("Failed to process request with middleware: {0}")]
    MiddlewareError(#[from] anyhow::Error),

    #[error("Failed to verify hash: {actual} != {expected}")]
    HashError { actual: String, expected: String },

    #[error("Failed to parse JSON")]
    JsonError(#[from] serde_json::Error),

    #[error("Failed to parse Toml")]
    TomlError(#[from] toml::de::Error),

    #[error("Content is not UTF-8")]
    ParseError(#[from] Utf8Error),
}
