use serr::SerializeError;

use crate::{
    core::WindowError, features::settings::AppSettingsError, shared::IdempotencyManagerError,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, SerializeError)]
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
