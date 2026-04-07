use std::path::PathBuf;

use serr::SerializeError;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum FileWatcherError {
    #[error("Path not found: {path}")]
    PathNotFound { path: PathBuf },

    #[error("Attempted to remove a watch that does not exist")]
    WatchNotFound,

    #[error("File watching error: {0}")]
    NotifyError(#[from] notify::Error),
}
