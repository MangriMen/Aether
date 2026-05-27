use async_trait::async_trait;
use std::path::Path;

use crate::features::file_watcher::FileWatcherError;

#[async_trait]
pub trait FileWatcher: Send + Sync {
    async fn watch(&self, path: &Path) -> Result<(), FileWatcherError>;
    async fn unwatch(&self, path: &Path) -> Result<(), FileWatcherError>;
}
