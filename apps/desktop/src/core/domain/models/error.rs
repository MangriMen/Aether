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

    #[test]
    fn error_launch_error_format() {
        let err = Error::LaunchError("port in use".to_string());
        assert_eq!(err.to_string(), "Launch critical error: port in use");
    }

    #[test]
    fn error_from_app_settings_error() {
        let app_err = AppSettingsError::Storage("db error".to_string());
        let err: Error = app_err.into();
        assert_eq!(err.to_string(), "Storage error: db error");
    }

    #[test]
    fn error_from_window_error() {
        let window_err = WindowError::Other("oops".to_string());
        let err: Error = window_err.into();
        assert_eq!(err.to_string(), "Unexpected error: oops");
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
