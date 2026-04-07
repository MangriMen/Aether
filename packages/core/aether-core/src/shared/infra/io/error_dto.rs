use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum IoErrorKind {
    NotFound,
    PermissionDenied,
    Other,
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum IoErrorRepr {
    #[error("{kind:?}, path: {path}")]
    IoPathError {
        kind: IoErrorKind,
        message: String,
        path: String,
    },

    #[error("{message:?}")]
    IoError { kind: IoErrorKind, message: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl From<std::io::ErrorKind> for IoErrorKind {
    fn from(value: std::io::ErrorKind) -> Self {
        match value {
            std::io::ErrorKind::NotFound => Self::NotFound,
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            _ => Self::Other,
        }
    }
}

impl From<IoErrorKind> for std::io::ErrorKind {
    fn from(value: IoErrorKind) -> Self {
        match value {
            IoErrorKind::NotFound => Self::NotFound,
            IoErrorKind::PermissionDenied => Self::PermissionDenied,
            IoErrorKind::Other => Self::Other,
        }
    }
}
