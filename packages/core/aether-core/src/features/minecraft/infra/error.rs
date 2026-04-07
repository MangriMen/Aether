use crate::{features::minecraft::MinecraftDomainError, shared::IoError};

impl From<IoError> for MinecraftDomainError {
    fn from(value: IoError) -> Self {
        MinecraftDomainError::StorageFailure {
            reason: value.to_string(),
        }
    }
}
