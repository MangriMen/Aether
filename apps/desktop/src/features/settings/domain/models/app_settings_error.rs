use aether_core::shared::error::DatabaseError;
use aether_core::shared::io::IoError;

use crate::core::WindowError;

#[derive(Debug, thiserror::Error)]
pub enum AppSettingsError {
    #[error("Can't set effect: {0}")]
    CanNotSetEffect(#[source] WindowError),

    #[error("Can't recreate window: {0}")]
    CanNotRecreateWindow(#[source] WindowError),

    #[error("Storage error: {0}")]
    Storage(#[from] AppSettingsStorageError),
}

/// Kept typed so the frontend gets a category instead of the raw storage message.
#[derive(Debug, thiserror::Error)]
pub enum AppSettingsStorageError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Database(#[from] DatabaseError),
}

impl From<IoError> for AppSettingsError {
    fn from(err: IoError) -> Self {
        Self::Storage(err.into())
    }
}

impl From<DatabaseError> for AppSettingsError {
    fn from(err: DatabaseError) -> Self {
        Self::Storage(err.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::WindowLabel;
    use rstest::rstest;

    #[rstest]
    #[case::cannot_set_effect(
        AppSettingsError::CanNotSetEffect(WindowError::PlatformNotSupported),
        "Can't set effect: The platform does not support this"
    )]
    #[case::cannot_recreate_window(
        AppSettingsError::CanNotRecreateWindow(WindowError::NotFound {
            label: WindowLabel::Main
        }),
        "Can't recreate window: Window with label 'main' was not found"
    )]
    #[case::storage(
        AppSettingsError::from(DatabaseError::new("pool timed out")),
        "Storage error: Database error: pool timed out"
    )]
    fn error_format(#[case] err: AppSettingsError, #[case] expected: &str) {
        assert_eq!(err.to_string(), expected);
    }
}
