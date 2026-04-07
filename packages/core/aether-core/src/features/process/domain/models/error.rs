use serr::SerializeError;

use crate::shared::IoError;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum ProcessError {
    #[error("Failed to kill process {id}")]
    KillError { id: String },

    #[error("Failed to wait process {id}")]
    WaitError { id: String },

    #[error(transparent)]
    Io(#[from] IoError),
}
