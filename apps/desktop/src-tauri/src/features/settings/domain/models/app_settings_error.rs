use serr::SerializeError;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum AppSettingsError {
    #[error("Can't set effect: {0}")]
    CanNotSetEffect(String),

    #[error("Can't recreate window: {0}")]
    CanNotRecreateWindow(String),

    #[error("Failed to save settings")]
    SaveError,
}
