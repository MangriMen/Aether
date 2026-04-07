use serr::SerializeError;

use crate::{
    features::{java::app::JavaApplicationError, minecraft::MinecraftDomainError},
    shared::IoError,
};

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum MinecraftApplicationError {
    #[error(transparent)]
    Domain(#[from] MinecraftDomainError),

    #[error(transparent)]
    JavaError(#[from] JavaApplicationError),

    #[error("Storage operation failed: {0}")]
    Storage(#[from] IoError),
}
