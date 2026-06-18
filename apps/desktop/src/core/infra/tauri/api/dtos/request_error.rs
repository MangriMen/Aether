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
        }
    }
}
