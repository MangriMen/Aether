use std::{collections::HashSet, path::Path};

use async_trait::async_trait;

use crate::features::{
    events::{EventEmitterExt, InstanceEvent, InstanceEventType, SharedEventEmitter},
    file_watcher::{FileEvent, FileEventHandler, FileWatcherError},
    instance::InstanceId,
    settings::INSTANCES_FOLDER_NAME,
};

/// Tells the frontend which instances changed on disk. A crash of the game is not looked for
/// here: the frontend shows it from the process that ended.
pub struct InstanceEventHandler {
    event_emitter: SharedEventEmitter,
}

impl InstanceEventHandler {
    pub fn new(event_emitter: SharedEventEmitter) -> Self {
        Self { event_emitter }
    }

    fn extract_instance_path(path: &Path) -> Option<InstanceId> {
        let mut found = false;
        for component in path.components() {
            if found {
                return InstanceId::try_from(component.as_os_str().to_string_lossy().to_string())
                    .ok();
            }
            if component.as_os_str() == INSTANCES_FOLDER_NAME {
                found = true;
            }
        }
        None
    }
}

#[async_trait]
impl FileEventHandler for InstanceEventHandler {
    async fn handle_events(
        &self,
        events: Result<Vec<FileEvent>, FileWatcherError>,
    ) -> Result<(), FileWatcherError> {
        let events = match events {
            Ok(events) => events,
            Err(error) => {
                tracing::warn!("File watcher reported an error: {error}");
                return Ok(());
            }
        };

        // One `Synced` event per instance per batch, however many files changed.
        let mut synced: HashSet<InstanceId> = HashSet::new();

        for event in &events {
            let Some(instance_id) = Self::extract_instance_path(&event.path) else {
                continue;
            };

            if synced.insert(instance_id.clone()) {
                let event_emitter = self.event_emitter.clone();
                tokio::spawn(async move {
                    event_emitter
                        .emit_safe(InstanceEvent {
                            event: InstanceEventType::Synced,
                            instance_id,
                        })
                        .await;
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::sync::mpsc;

    use crate::features::events::{Event, EventEmitter, EventError};
    use crate::features::file_watcher::FileEventKind;

    use super::*;

    struct ChannelEmitter(mpsc::UnboundedSender<Event>);

    #[async_trait]
    impl EventEmitter<Event> for ChannelEmitter {
        async fn emit(&self, event: Event) -> Result<(), EventError> {
            let _ = self.0.send(event);
            Ok(())
        }

        fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
    }

    fn handler() -> (InstanceEventHandler, mpsc::UnboundedReceiver<Event>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let handler = InstanceEventHandler::new(Arc::new(ChannelEmitter(tx)));
        (handler, rx)
    }

    fn modified(path: &str) -> FileEvent {
        FileEvent {
            kind: FileEventKind::Modify,
            path: PathBuf::from(path),
        }
    }

    async fn next_event(rx: &mut mpsc::UnboundedReceiver<Event>) -> Event {
        tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("timed out waiting for an event")
            .expect("event channel closed")
    }

    /// Lets every spawned task run to completion, then checks that nothing was emitted.
    async fn assert_silent(rx: &mut mpsc::UnboundedReceiver<Event>) {
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }
        assert!(rx.try_recv().is_err(), "unexpected event");
    }

    #[tokio::test]
    async fn emits_one_synced_event_per_instance_in_a_batch() {
        let (handler, mut rx) = handler();

        handler
            .handle_events(Ok(vec![
                modified("/data/instances/abc/mods/a.jar"),
                modified("/data/instances/abc/mods/b.jar"),
                modified("/data/instances/xyz/mods/c.jar"),
            ]))
            .await
            .unwrap();

        let mut ids = Vec::new();
        for _ in 0..2 {
            let Event::Instance(event) = next_event(&mut rx).await else {
                panic!("expected instance event");
            };
            assert!(matches!(event.event, InstanceEventType::Synced));
            ids.push(event.instance_id);
        }
        ids.sort();
        assert_eq!(
            ids,
            vec![
                InstanceId::try_from("abc").unwrap(),
                InstanceId::try_from("xyz").unwrap()
            ]
        );
        assert_silent(&mut rx).await;
    }

    #[tokio::test]
    async fn ignores_paths_outside_instances_folder() {
        let (handler, mut rx) = handler();

        handler
            .handle_events(Ok(vec![modified("/data/other/abc/file.txt")]))
            .await
            .unwrap();

        assert_silent(&mut rx).await;
    }

    #[tokio::test]
    async fn should_only_sync_the_instance_when_a_crash_report_appears() {
        let (handler, mut rx) = handler();

        handler
            .handle_events(Ok(vec![modified(
                "/data/instances/abc/crash-reports/crash-1.txt",
            )]))
            .await
            .unwrap();

        let Event::Instance(event) = next_event(&mut rx).await else {
            panic!("expected instance event");
        };
        assert!(matches!(event.event, InstanceEventType::Synced));
        assert_silent(&mut rx).await;
    }

    #[tokio::test]
    async fn watcher_error_is_swallowed() {
        let (handler, mut rx) = handler();

        let result = handler
            .handle_events(Err(FileWatcherError::WatchNotFound))
            .await;

        assert!(result.is_ok());
        assert_silent(&mut rx).await;
    }
}
