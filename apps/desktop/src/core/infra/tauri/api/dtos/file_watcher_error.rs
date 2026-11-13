use aether_core::features::file_watcher::FileWatcherError;
use serde::Serialize;
use specta::Type;

use super::CauseDto;

#[derive(Clone, Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileWatcherErrorDto {
    PathNotFound { path: String },
    WatchNotFound,
    NotifyError { cause: CauseDto },
}

impl From<&FileWatcherError> for FileWatcherErrorDto {
    fn from(value: &FileWatcherError) -> Self {
        match value {
            FileWatcherError::PathNotFound { path } => Self::PathNotFound {
                path: path.to_string_lossy().to_string(),
            },
            FileWatcherError::WatchNotFound => Self::WatchNotFound,
            FileWatcherError::NotifyError(err) => Self::NotifyError {
                cause: CauseDto::unknown(err),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn should_serialize_notify_error_with_an_unknown_cause() {
        let dto = FileWatcherErrorDto::from(&FileWatcherError::NotifyError("boom".to_owned()));

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({"code": "NOTIFY_ERROR", "payload": {"cause": {"type": "unknown", "details": "boom"}}})
        );
    }
}
