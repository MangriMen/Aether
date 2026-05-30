use crate::{
    features::{java::JavaApplicationError, minecraft::MinecraftDomainError},
    shared::io::domain::IoError,
};

#[derive(Debug, thiserror::Error)]
pub enum MinecraftApplicationError {
    #[error(transparent)]
    Domain(#[from] MinecraftDomainError),

    #[error(transparent)]
    JavaError(#[from] JavaApplicationError),

    #[error("Storage operation failed: {0}")]
    Storage(#[from] IoError),
}
