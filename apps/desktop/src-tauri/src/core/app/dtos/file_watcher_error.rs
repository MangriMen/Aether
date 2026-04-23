use aether_core::features::file_watcher::FileWatcherError;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileWatcherErrorDto {
    PathNotFound { path: String },
    WatchNotFound,
    NotifyError { details: String },
}

impl From<&FileWatcherError> for FileWatcherErrorDto {
    fn from(value: &FileWatcherError) -> Self {
        match value {
            FileWatcherError::PathNotFound { path } => Self::PathNotFound {
                path: path.to_string_lossy().to_string(),
            },
            FileWatcherError::WatchNotFound => Self::WatchNotFound,
            FileWatcherError::NotifyError(err) => Self::NotifyError {
                details: err.to_string(),
            },
        }
    }
}
