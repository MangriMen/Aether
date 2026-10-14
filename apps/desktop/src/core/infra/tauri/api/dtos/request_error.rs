use aether_core::shared::request_client::RequestError;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestErrorDto {
    Acquire,
    RequestSend { details: String },
    Middleware { details: String },
    Hash { actual: String, expected: String },
    Json { details: String },
    Toml { details: String },
    Parse,
    Status { status: u16, url: String },
    // `String`, not `u64`: specta refuses to export `u64` to TypeScript.
    TooLarge { limit: String },
    Io { path: String, details: String },
}

impl From<&RequestError> for RequestErrorDto {
    fn from(value: &RequestError) -> Self {
        match value {
            RequestError::Acquire(_) => Self::Acquire,
            RequestError::RequestSend(err) => Self::RequestSend {
                details: err.to_string(),
            },
            RequestError::Middleware(err) => Self::Middleware {
                details: err.to_string(),
            },
            RequestError::Hash { actual, expected } => Self::Hash {
                actual: actual.clone(),
                expected: expected.clone(),
            },
            RequestError::Json(err) => Self::Json {
                details: err.to_string(),
            },
            RequestError::Toml(err) => Self::Toml {
                details: err.to_string(),
            },
            RequestError::Parse(_) => Self::Parse,
            RequestError::Status { status, url } => Self::Status {
                status: *status,
                url: url.clone(),
            },
            RequestError::TooLarge { limit } => Self::TooLarge {
                limit: limit.to_string(),
            },
            RequestError::Io { path, source } => Self::Io {
                path: path.display().to_string(),
                details: source.to_string(),
            },
        }
    }
}
