use aether_core::shared::io::IoError;

#[derive(Debug, thiserror::Error)]
pub enum AppSettingsError {
    #[error("Can't set effect: {0}")]
    CanNotSetEffect(String),

    #[error("Can't recreate window: {0}")]
    CanNotRecreateWindow(String),

    #[error("Storage error: {0}")]
    Storage(#[from] AppSettingsStorageError),
}

/// Kept typed so the frontend gets a category instead of the raw storage message.
#[derive(Debug, thiserror::Error)]
pub enum AppSettingsStorageError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl From<IoError> for AppSettingsError {
    fn from(err: IoError) -> Self {
        Self::Storage(err.into())
    }
}

impl From<sqlx::Error> for AppSettingsError {
    fn from(err: sqlx::Error) -> Self {
        Self::Storage(err.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::cannot_set_effect(
        AppSettingsError::CanNotSetEffect("not supported".to_string()),
        "Can't set effect: not supported"
    )]
    #[case::cannot_recreate_window(
        AppSettingsError::CanNotRecreateWindow("window busy".to_string()),
        "Can't recreate window: window busy"
    )]
    #[case::storage(
        AppSettingsError::from(sqlx::Error::PoolTimedOut),
        "Storage error: pool timed out while waiting for an open connection"
    )]
    fn error_format(#[case] err: AppSettingsError, #[case] expected: &str) {
        assert_eq!(err.to_string(), expected);
    }
}
