use std::sync::Arc;

use crate::features::file_watcher::app::{FileEventHandler, FileWatcher};

/// Extension trait providing access to all file watcher feature ports.
///
/// Implemented on the core dependency injection container to expose
/// file-watcher-specific functionality in a centralized manner.
///
/// File watcher is not exposed directly — use `InstanceWatcherService` instead.
pub trait FileWatcherFeature {
    fn file_watcher(&self) -> Option<Arc<dyn FileWatcher>>;
    fn file_event_handler(&self) -> Option<Arc<dyn FileEventHandler>>;
}
