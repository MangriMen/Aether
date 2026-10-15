use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum FileWatcherError {
    #[error("Path not found: {path}")]
    PathNotFound { path: PathBuf },

    #[error("Attempted to remove a watch that does not exist")]
    WatchNotFound,

    #[error("File watching error: {0}")]
    NotifyError(String),
}
