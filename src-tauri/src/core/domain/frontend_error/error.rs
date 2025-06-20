use std::borrow::Cow;

use serde::Serialize;
use serializable_error::ToSerializableError;

pub type FrontendResult<T, E = FrontendError> = Result<T, E>;

#[derive(Debug, Serialize)]
pub struct FrontendError {
    pub code: Cow<'static, str>,
    pub fields: Option<serde_json::Value>,
    pub message: String,
}

impl From<aether_core::Error> for FrontendError {
    fn from(value: aether_core::Error) -> Self {
        let serialized = value.raw.to_serializable();
        Self {
            code: serialized.code,
            fields: serialized.fields,
            message: serialized.message,
        }
    }
}

impl From<crate::Error> for FrontendError {
    fn from(value: crate::Error) -> Self {
        let serialized = value.to_serializable();
        Self {
            code: serialized.code,
            fields: serialized.fields,
            message: serialized.message,
        }
    }
}
