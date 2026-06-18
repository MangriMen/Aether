#[derive(Debug, thiserror::Error)]
pub enum AppSettingsError {
    #[error("Can't set effect: {0}")]
    CanNotSetEffect(String),

    #[error("Can't recreate window: {0}")]
    CanNotRecreateWindow(String),

    #[error("Storage error: {0}")]
    Storage(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cannot_set_effect_format() {
        let err = AppSettingsError::CanNotSetEffect("not supported".to_string());
        assert_eq!(err.to_string(), "Can't set effect: not supported");
    }

    #[test]
    fn cannot_recreate_window_format() {
        let err = AppSettingsError::CanNotRecreateWindow("window busy".to_string());
        assert_eq!(err.to_string(), "Can't recreate window: window busy");
    }

    #[test]
    fn storage_error_format() {
        let err = AppSettingsError::Storage("disk full".to_string());
        assert_eq!(err.to_string(), "Storage error: disk full");
    }
}
