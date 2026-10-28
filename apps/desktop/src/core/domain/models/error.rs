use crate::{
    core::WindowError, features::settings::AppSettingsError, shared::IdempotencyManagerError,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error(transparent)]
    AppSettingsError(#[from] AppSettingsError),

    #[error(transparent)]
    WindowError(#[from] WindowError),

    #[error("Launch critical error: {0}")]
    LaunchError(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    IdempotencyManager(#[from] IdempotencyManagerError),

    #[error(transparent)]
    Core(#[from] aether_core::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::RequestId;
    use rstest::rstest;

    #[rstest]
    #[case::launch_error(
        Error::LaunchError("port in use".to_string()),
        "Launch critical error: port in use"
    )]
    #[case::from_app_settings_error(
        AppSettingsError::Storage("db error".to_string()).into(),
        "Storage error: db error"
    )]
    #[case::from_window_error(
        WindowError::Other("oops".to_string()).into(),
        "Unexpected error: oops"
    )]
    fn error_display(#[case] err: Error, #[case] expected: &str) {
        assert_eq!(err.to_string(), expected);
    }

    #[test]
    fn error_from_idempotency_error() {
        let idem_err = IdempotencyManagerError::DuplicateRequest {
            id: RequestId::new("test".to_string()),
        };
        let err: Error = idem_err.into();
        assert!(
            err.to_string().contains("Duplicate request"),
            "Expected Duplicate request error, got: {err}"
        );
    }
}
