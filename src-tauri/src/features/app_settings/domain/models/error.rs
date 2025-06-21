use serr::SerializeError;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum AppSettingsError {
    #[error("Transparent depend window effect can't be turned on without transparent effect")]
    TransparentEffectIsRequired,

    #[error("Can't set error: {0}")]
    CanNotSetEffect(String),
}
