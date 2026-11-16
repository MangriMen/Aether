use crate::{
    features::{java::JavaApplicationError, minecraft::MinecraftDomainError},
    shared::{io::domain::IoError, serializable_command::HookSpawnError},
};

#[derive(Clone, Debug, thiserror::Error)]
pub enum MinecraftApplicationError {
    #[error(transparent)]
    Domain(#[from] MinecraftDomainError),

    #[error(transparent)]
    JavaError(#[from] JavaApplicationError),

    #[error("Storage operation failed: {0}")]
    Storage(#[from] IoError),

    /// The wrapper command of the user is not a command.
    #[error(transparent)]
    Hook(#[from] HookSpawnError),
}
