use std::{path::PathBuf, str::Utf8Error, sync::Arc};

use crate::shared::error::AnyhowSource;

/// Clones share the library errors they hold, which are not `Clone` themselves.
#[derive(Clone, thiserror::Error, Debug)]
pub enum RequestError {
    #[error("Failed to acquire semaphore")]
    Acquire(#[source] Arc<tokio::sync::AcquireError>),

    #[error("Failed to send request: {0}")]
    RequestSend(#[source] Arc<reqwest::Error>),

    #[error("Failed to process request with middleware: {0}")]
    Middleware(#[source] AnyhowSource),

    #[error("Failed to verify hash: {actual} != {expected}")]
    Hash { actual: String, expected: String },

    #[error("Failed to parse JSON: {0}")]
    Json(#[source] Arc<serde_json::Error>),

    #[error("Failed to parse Toml: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Content is not UTF-8")]
    Parse(#[from] Utf8Error),

    #[error("Unexpected HTTP status {status} from `{url}`")]
    Status { status: u16, url: String },

    #[error("Response body exceeds the {limit} byte limit")]
    TooLarge { limit: u64 },

    #[error("Failed to write download to `{}`: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: Arc<std::io::Error>,
    },
}

impl From<tokio::sync::AcquireError> for RequestError {
    fn from(err: tokio::sync::AcquireError) -> Self {
        Self::Acquire(Arc::new(err))
    }
}

impl From<reqwest::Error> for RequestError {
    fn from(err: reqwest::Error) -> Self {
        Self::RequestSend(Arc::new(err))
    }
}

impl From<anyhow::Error> for RequestError {
    fn from(err: anyhow::Error) -> Self {
        Self::Middleware(err.into())
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(Arc::new(err))
    }
}
