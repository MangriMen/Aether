use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::shared::io::error_dto::IoErrorRepr;

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("{source}, path: {path}")]
    IoPathError {
        #[source]
        source: std::io::Error,
        path: String,
    },
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl IoError {
    pub fn from(source: std::io::Error) -> Self {
        Self::IoError(source)
    }

    pub fn with_path(source: std::io::Error, path: impl AsRef<Path>) -> Self {
        Self::IoPathError {
            source,
            path: path.as_ref().to_string_lossy().to_string(),
        }
    }
}

impl From<&IoError> for IoErrorRepr {
    fn from(value: &IoError) -> Self {
        match value {
            IoError::IoPathError { source, path } => Self::IoPathError {
                kind: source.kind().into(),
                path: path.clone(),
                message: source.to_string(),
            },
            IoError::IoError(source) => Self::IoError {
                kind: source.kind().into(),
                message: source.to_string(),
            },
            IoError::SerializationError(msg) => Self::SerializationError(msg.to_owned()),
            IoError::DeserializationError(msg) => Self::DeserializationError(msg.to_owned()),
        }
    }
}

impl From<IoErrorRepr> for IoError {
    fn from(value: IoErrorRepr) -> Self {
        match value {
            IoErrorRepr::IoPathError {
                kind,
                path,
                message,
            } => Self::IoPathError {
                source: std::io::Error::new(kind.into(), message.to_owned()),
                path,
            },
            IoErrorRepr::IoError { kind, message } => {
                Self::IoError(std::io::Error::new(kind.into(), message.to_owned()))
            }

            IoErrorRepr::SerializationError(msg) => Self::SerializationError(msg),

            IoErrorRepr::DeserializationError(msg) => Self::DeserializationError(msg),
        }
    }
}

impl Serialize for IoError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let repr: IoErrorRepr = self.into();
        repr.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for IoError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let repr = IoErrorRepr::deserialize(deserializer)?;
        Ok(repr.into())
    }
}
