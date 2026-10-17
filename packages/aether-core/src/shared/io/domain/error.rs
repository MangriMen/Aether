use serde::{Deserialize, Serialize};
use std::path::Path;

// --- IoErrorKind ---

#[derive(Debug, Serialize, Deserialize)]
pub enum IoErrorKind {
    NotFound,
    PermissionDenied,
    Other,
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

// --- IoErrorRepr ---

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

// --- IoError ---

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
    /// Wraps an OS error together with the path it happened on.
    ///
    /// For an error without a meaningful path use `IoError::from` (the derived
    /// `From<std::io::Error>` conversion).
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
                source: std::io::Error::new(kind.into(), message.clone()),
                path,
            },
            IoErrorRepr::IoError { kind, message } => {
                Self::IoError(std::io::Error::new(kind.into(), message.clone()))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn not_found() -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::NotFound, "gone")
    }

    #[test]
    fn plain_error_displays_source_transparently() {
        let err = IoError::from(not_found());
        assert!(matches!(err, IoError::IoError(_)));
        assert_eq!(err.to_string(), "gone");
    }

    #[test]
    fn path_error_displays_source_and_path() {
        let err = IoError::with_path(not_found(), "/tmp/x");
        assert!(matches!(err, IoError::IoPathError { .. }));
        assert_eq!(err.to_string(), "gone, path: /tmp/x");
    }

    #[test]
    fn serialization_variants_display() {
        assert_eq!(
            IoError::SerializationError("a".into()).to_string(),
            "Serialization error: a"
        );
        assert_eq!(
            IoError::DeserializationError("b".into()).to_string(),
            "Deserialization error: b"
        );
    }

    #[test]
    fn serde_round_trip_keeps_kind_path_and_message() {
        let err = IoError::with_path(not_found(), "/tmp/x");
        let json = serde_json::to_string(&err).unwrap();
        let back: IoError = serde_json::from_str(&json).unwrap();

        let IoError::IoPathError { source, path } = back else {
            panic!("variant changed");
        };
        assert_eq!(path, "/tmp/x");
        assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
        assert_eq!(source.to_string(), "gone");
    }

    #[test]
    fn unknown_kinds_collapse_to_other() {
        let err = IoError::from(std::io::Error::new(std::io::ErrorKind::TimedOut, "slow"));
        let json = serde_json::to_string(&err).unwrap();
        let IoError::IoError(source) = serde_json::from_str::<IoError>(&json).unwrap() else {
            panic!("variant changed");
        };
        assert_eq!(source.kind(), std::io::ErrorKind::Other);
    }
}
