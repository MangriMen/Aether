use serr::SerializeError;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum AppSettingsError {
    #[error("Transparent depend window effect can't be turned on without transparent effect")]
    TransparentEffectRequired,

    #[error("Can't set effect: {0}")]
    CanNotSetEffect(String),

    #[error("Can't recreate window: {0}")]
    CanNotRecreateWindow(String),

    #[error("Failed to save settings")]
    SaveError,
}
