use aether_core::libs::request_client::RequestError;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestErrorDto {
    AcquireError,
    RequestSendError { details: String },
    MiddlewareError { details: String },
    HashError { actual: String, expected: String },
    JsonError { details: String },
    TomlError { details: String },
    ParseError,
}

impl From<&RequestError> for RequestErrorDto {
    fn from(value: &RequestError) -> Self {
        match value {
            RequestError::AcquireError(_) => Self::AcquireError,
            RequestError::RequestSendError(err) => Self::RequestSendError {
                details: err.to_string(),
            },
            RequestError::MiddlewareError(err) => Self::MiddlewareError {
                details: err.to_string(),
            },
            RequestError::HashError { actual, expected } => Self::HashError {
                actual: actual.clone(),
                expected: expected.clone(),
            },
            RequestError::JsonError(err) => Self::JsonError {
                details: err.to_string(),
            },
            RequestError::TomlError(err) => Self::TomlError {
                details: err.to_string(),
            },
            RequestError::ParseError(_) => Self::ParseError,
        }
    }
}
