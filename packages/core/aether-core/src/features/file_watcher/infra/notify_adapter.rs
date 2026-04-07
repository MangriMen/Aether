use std::{path::Path, sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use tokio::sync::RwLock;

use crate::features::file_watcher::{
    FileEvent, FileEventHandler, FileEventKind, FileWatcher, FileWatcherError,
};

pub struct NotifyFileWatcher<FEH: FileEventHandler> {
    watcher: RwLock<Debouncer<RecommendedWatcher>>,
    _event_handler: Arc<FEH>,
}

impl<FEH: FileEventHandler + 'static> NotifyFileWatcher<FEH> {
    pub fn new(event_handler: Arc<FEH>) -> Result<Self, FileWatcherError> {
        let (mut tx, rx) = channel(1);

        let watcher = new_debouncer(
            Duration::from_secs_f32(1.0),
            move |res: DebounceEventResult| {
                futures::executor::block_on(async {
                    if let Err(e) = tx.send(res).await {
                        tracing::error!("Failed to send file event: {}", e);
                    };
                })
            },
        )?;

        Self::spawn_event_processor(rx, event_handler.clone());

        Ok(Self {
            watcher: RwLock::new(watcher),
            _event_handler: event_handler,
        })
    }

    fn spawn_event_processor(mut rx: Receiver<DebounceEventResult>, event_handler: Arc<FEH>) {
        tokio::spawn(async move {
            let span = tracing::span!(tracing::Level::DEBUG, "file_watcher");
            tracing::debug!(parent: &span, "Starting file watcher event processor");

            while let Some(res) = rx.next().await {
                let events = res
                    .map(|events| {
                        events
                            .into_iter()
                            .map(|event| FileEvent {
                                kind: FileEventKind::Modify,
                                path: event.path,
                            })
                            .collect()
                    })
                    .map_err(FileWatcherError::from);

                if let Err(e) = event_handler.handle_events(events).await {
                    tracing::error!(parent: &span, "Failed to handle file events: {}", e);
                }
            }
        });
    }
}

#[async_trait]
impl<FEH: FileEventHandler> FileWatcher for NotifyFileWatcher<FEH> {
    async fn watch(&self, path: &Path) -> Result<(), FileWatcherError> {
        let mut watcher = self.watcher.write().await;
        watcher.watcher().watch(path, RecursiveMode::Recursive)?;
        Ok(())
    }

    async fn unwatch(&self, path: &Path) -> Result<(), FileWatcherError> {
        let mut watcher = self.watcher.write().await;
        watcher.watcher().unwatch(path)?;
        Ok(())
    }
}
