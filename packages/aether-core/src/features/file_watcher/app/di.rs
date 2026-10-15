use std::sync::Arc;

use crate::features::file_watcher::app::{FileEventHandler, FileWatcher};

// TODO: Implement FileWatcherFeature on AetherContainer.
// Raw file-watcher access is not currently exposed through the container;
// use `InstanceWatcherService` instead for instance-level file watching.
// When implementing, add an `impl FileWatcherFeature for AetherContainer` block
// in `core/app/di.rs` providing the concrete `NotifyFileWatcher` and handler.
pub trait FileWatcherFeature {
    fn file_watcher(&self) -> Option<Arc<dyn FileWatcher>>;
    fn file_event_handler(&self) -> Option<Arc<dyn FileEventHandler>>;
}
