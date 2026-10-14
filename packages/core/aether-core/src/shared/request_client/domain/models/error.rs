use std::{path::PathBuf, str::Utf8Error};

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("Failed to acquire semaphore")]
    Acquire(#[from] tokio::sync::AcquireError),

    #[error("Failed to send request: {0}")]
    RequestSend(#[from] reqwest::Error),

    #[error("Failed to process request with middleware: {0}")]
    Middleware(#[from] anyhow::Error),

    #[error("Failed to verify hash: {actual} != {expected}")]
    Hash { actual: String, expected: String },

    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

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
        source: std::io::Error,
    },
}
