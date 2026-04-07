use serr::SerializeError;
use uuid::Uuid;

use super::progress_bar_error::ProgressBarStorageError;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum EventError {
    #[error("Event state was not properly initialized")]
    NotInitialized,

    #[error("Non-existent loading bar of key: {0}")]
    NoLoadingBar(Uuid),

    #[error("Failed to sent event")]
    SerializeError(anyhow::Error),

    #[error("Storage error: {0}")]
    StorageError(#[from] ProgressBarStorageError),
}
