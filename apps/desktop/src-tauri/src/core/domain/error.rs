use serr::SerializeError;

use crate::features::settings::AppSettingsError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, SerializeError)]
pub enum Error {
    #[error(transparent)]
    AppSettingsError(#[from] AppSettingsError),

    #[error("Launch critical error: {0}")]
    LaunchError(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    Core(#[from] aether_core::Error),
}
