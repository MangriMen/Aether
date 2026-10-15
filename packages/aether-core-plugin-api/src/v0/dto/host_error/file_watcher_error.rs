use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum FileWatcherErrorDto {
    PathNotFound { path: String },
    WatchNotFound,
    NotifyError(String),
}
