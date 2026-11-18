use std::{collections::HashSet, path::Path, sync::Arc};

use async_trait::async_trait;

use crate::features::{
    events::{EventEmitterExt, InstanceEvent, InstanceEventType, SharedEventEmitter},
    file_watcher::{FileEvent, FileEventHandler, FileWatcherError},
    instance::InstanceId,
    settings::LocationInfo,
};

/// Tells the frontend which instances changed on disk. A crash of the game is not looked for
/// here: the frontend shows it from the process that ended.
pub struct InstanceEventHandler {
    event_emitter: SharedEventEmitter,
    location_info: Arc<LocationInfo>,
}

impl InstanceEventHandler {
    pub fn new(event_emitter: SharedEventEmitter, location_info: Arc<LocationInfo>) -> Self {
        Self {
            event_emitter,
            location_info,
        }
    }

    /// The instance whose folder holds `path`: the first component below the instances
    /// directory, so a data directory that itself has an `instances` folder in its path does
    /// not mislead.
    fn extract_instance_path(&self, path: &Path) -> Option<InstanceId> {
        let relative = path.strip_prefix(self.location_info.instances_dir()).ok()?;
        let first = relative.components().next()?;
        InstanceId::try_from(first.as_os_str().to_string_lossy().to_string()).ok()
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
            let Some(instance_id) = self.extract_instance_path(&event.path) else {
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
        handler_in("/data")
    }

    fn handler_in(config_dir: &str) -> (InstanceEventHandler, mpsc::UnboundedReceiver<Event>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let location_info = Arc::new(LocationInfo::new(
            PathBuf::from("/settings"),
            PathBuf::from(config_dir),
        ));
        let handler = InstanceEventHandler::new(Arc::new(ChannelEmitter(tx)), location_info);
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
    async fn should_name_the_instance_when_the_data_dir_has_an_instances_folder_in_its_path() {
        let (handler, mut rx) = handler_in("/instances/Aether");

        handler
            .handle_events(Ok(vec![modified(
                "/instances/Aether/instances/abc/mods/a.jar",
            )]))
            .await
            .unwrap();

        let Event::Instance(event) = next_event(&mut rx).await else {
            panic!("expected instance event");
        };
        assert_eq!(event.instance_id, InstanceId::try_from("abc").unwrap());
        assert_silent(&mut rx).await;
    }

    #[tokio::test]
    async fn should_ignore_a_path_that_is_not_below_an_instance() {
        let (handler, mut rx) = handler();

        handler
            .handle_events(Ok(vec![
                modified("/data/instances"),
                modified("/other/instances/abc/mods/a.jar"),
            ]))
            .await
            .unwrap();

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
