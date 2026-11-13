use uuid::Uuid;

use super::progress_bar_error::ProgressBarStorageError;
use crate::shared::error::AnyhowSource;

#[derive(Clone, Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event state was not properly initialized")]
    NotInitialized,

    #[error("Non-existent loading bar of key: {0}")]
    NoLoadingBar(Uuid),

    #[error("Failed to sent event")]
    SerializeError(#[source] AnyhowSource),

    #[error("Storage error: {0}")]
    StorageError(#[from] ProgressBarStorageError),
}
