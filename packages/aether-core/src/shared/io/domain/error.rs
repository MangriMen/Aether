use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

// --- IoErrorKind ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoErrorKind {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    StorageFull,
    ReadOnlyFilesystem,
    ResourceBusy,
    TimedOut,
    InvalidData,
    Other,
}

impl IoErrorKind {
    /// Classifies an OS error, including Windows sharing/lock violations that std leaves uncategorized.
    pub fn of(error: &std::io::Error) -> Self {
        let kind = Self::from(error.kind());
        #[cfg(windows)]
        if kind == Self::Other
            && matches!(
                error.raw_os_error(),
                Some(ERROR_SHARING_VIOLATION | ERROR_LOCK_VIOLATION)
            )
        {
            return Self::ResourceBusy;
        }
        kind
    }
}

#[cfg(windows)]
const ERROR_SHARING_VIOLATION: i32 = 32;
#[cfg(windows)]
const ERROR_LOCK_VIOLATION: i32 = 33;

impl From<std::io::ErrorKind> for IoErrorKind {
    fn from(value: std::io::ErrorKind) -> Self {
        use std::io::ErrorKind as K;
        match value {
            K::NotFound => Self::NotFound,
            K::PermissionDenied => Self::PermissionDenied,
            K::AlreadyExists => Self::AlreadyExists,
            K::StorageFull | K::QuotaExceeded => Self::StorageFull,
            K::ReadOnlyFilesystem => Self::ReadOnlyFilesystem,
            K::ResourceBusy => Self::ResourceBusy,
            K::TimedOut => Self::TimedOut,
            K::InvalidData => Self::InvalidData,
            _ => Self::Other,
        }
    }
}

impl From<IoErrorKind> for std::io::ErrorKind {
    fn from(value: IoErrorKind) -> Self {
        match value {
            IoErrorKind::NotFound => Self::NotFound,
            IoErrorKind::PermissionDenied => Self::PermissionDenied,
            IoErrorKind::AlreadyExists => Self::AlreadyExists,
            IoErrorKind::StorageFull => Self::StorageFull,
            IoErrorKind::ReadOnlyFilesystem => Self::ReadOnlyFilesystem,
            IoErrorKind::ResourceBusy => Self::ResourceBusy,
            IoErrorKind::TimedOut => Self::TimedOut,
            IoErrorKind::InvalidData => Self::InvalidData,
            IoErrorKind::Other => Self::Other,
        }
    }
}

// --- IoErrorRepr ---

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error)]
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

#[derive(Clone, Debug, thiserror::Error)]
pub enum IoError {
    #[error("{source}, path: {path}")]
    IoPathError {
        #[source]
        source: Arc<std::io::Error>,
        path: String,
    },
    #[error(transparent)]
    IoError(Arc<std::io::Error>),

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
            source: Arc::new(source),
            path: path.as_ref().to_string_lossy().to_string(),
        }
    }
}

impl From<std::io::Error> for IoError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(Arc::new(err))
    }
}

impl From<&IoError> for IoErrorRepr {
    fn from(value: &IoError) -> Self {
        match value {
            IoError::IoPathError { source, path } => Self::IoPathError {
                kind: IoErrorKind::of(source),
                path: path.clone(),
                message: source.to_string(),
            },
            IoError::IoError(source) => Self::IoError {
                kind: IoErrorKind::of(source),
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
                source: Arc::new(std::io::Error::new(kind.into(), message.clone())),
                path,
            },
            IoErrorRepr::IoError { kind, message } => {
                Self::IoError(Arc::new(std::io::Error::new(kind.into(), message.clone())))
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
    fn path_error_displays_source_and_path() {
        let err = IoError::with_path(not_found(), "/tmp/x");
        assert!(matches!(err, IoError::IoPathError { .. }));
        assert_eq!(err.to_string(), "gone, path: /tmp/x");
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
        let err = IoError::from(std::io::Error::new(std::io::ErrorKind::Interrupted, "slow"));
        let json = serde_json::to_string(&err).unwrap();
        let IoError::IoError(source) = serde_json::from_str::<IoError>(&json).unwrap() else {
            panic!("variant changed");
        };
        assert_eq!(source.kind(), std::io::ErrorKind::Other);
    }

    #[rstest::rstest]
    #[case::not_found(std::io::ErrorKind::NotFound, IoErrorKind::NotFound)]
    #[case::already_exists(std::io::ErrorKind::AlreadyExists, IoErrorKind::AlreadyExists)]
    #[case::storage_full(std::io::ErrorKind::StorageFull, IoErrorKind::StorageFull)]
    #[case::quota(std::io::ErrorKind::QuotaExceeded, IoErrorKind::StorageFull)]
    #[case::read_only(
        std::io::ErrorKind::ReadOnlyFilesystem,
        IoErrorKind::ReadOnlyFilesystem
    )]
    #[case::busy(std::io::ErrorKind::ResourceBusy, IoErrorKind::ResourceBusy)]
    #[case::timed_out(std::io::ErrorKind::TimedOut, IoErrorKind::TimedOut)]
    #[case::invalid_data(std::io::ErrorKind::InvalidData, IoErrorKind::InvalidData)]
    #[case::other(std::io::ErrorKind::Interrupted, IoErrorKind::Other)]
    fn maps_std_kind(#[case] std_kind: std::io::ErrorKind, #[case] expected: IoErrorKind) {
        assert_eq!(IoErrorKind::from(std_kind), expected);
    }

    #[cfg(windows)]
    #[test]
    fn windows_sharing_violation_is_resource_busy() {
        assert_eq!(
            IoErrorKind::of(&std::io::Error::from_raw_os_error(32)),
            IoErrorKind::ResourceBusy
        );
        assert_eq!(
            IoErrorKind::of(&std::io::Error::from_raw_os_error(33)),
            IoErrorKind::ResourceBusy
        );
    }
}
