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
        AppSettingsError::Storage("disk full".to_string()),
        "Storage error: disk full"
    )]
    fn error_format(#[case] err: AppSettingsError, #[case] expected: &str) {
        assert_eq!(err.to_string(), expected);
    }
}
