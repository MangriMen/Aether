use std::{collections::HashSet, path::Path, sync::Arc};

use async_trait::async_trait;

use crate::features::{
    events::{EventEmitterExt, InstanceEvent, InstanceEventType, SharedEventEmitter, WarningEvent},
    file_watcher::{FileEvent, FileEventHandler, FileWatcherError},
    instance::{GetInstanceUseCasePort, InstanceInstallStage},
    settings::INSTANCES_FOLDER_NAME,
};

pub struct InstanceEventHandler {
    event_emitter: SharedEventEmitter,
    get_instance_uc: Arc<dyn GetInstanceUseCasePort>,
}

impl InstanceEventHandler {
    pub fn new(
        event_emitter: SharedEventEmitter,
        get_instance_uc: Arc<dyn GetInstanceUseCasePort>,
    ) -> Self {
        Self {
            event_emitter,
            get_instance_uc,
        }
    }

    fn extract_instance_path(path: &Path) -> Option<String> {
        let mut found = false;
        for component in path.components() {
            if found {
                return Some(component.as_os_str().to_string_lossy().to_string());
            }
            if component.as_os_str() == INSTANCES_FOLDER_NAME {
                found = true;
            }
        }
        None
    }

    fn is_crash_report(path: &Path) -> bool {
        path.components().any(|x| x.as_os_str() == "crash-reports")
            && path.extension().is_some_and(|x| x == "txt")
    }

    /// Looks the instance up in the background and, when it is fully installed, tells the
    /// frontend that a new crash report appeared. Half-installed instances stay silent,
    /// because a failed install is reported through its own channel.
    fn crash_task(&self, id: String) {
        let get_instance_uc = self.get_instance_uc.clone();
        let event_emitter = self.event_emitter.clone();

        tokio::spawn(async move {
            let Ok(instance) = get_instance_uc.execute(id).await else {
                return;
            };
            if instance.install_stage != InstanceInstallStage::Installed {
                return;
            }

            let message = format!(
                "Instance {} has crashed! Visit the logs page to see a crash report.",
                instance.name()
            );
            event_emitter.emit_safe(WarningEvent { message }).await;
        });
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
        let mut synced: HashSet<String> = HashSet::new();

        for event in &events {
            let Some(instance_id) = Self::extract_instance_path(&event.path) else {
                continue;
            };

            if Self::is_crash_report(&event.path) {
                self.crash_task(instance_id);
            } else if synced.insert(instance_id.clone()) {
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
    use std::time::Duration;

    use tokio::sync::mpsc;

    use crate::features::events::{Event, EventEmitter, EventError};
    use crate::features::file_watcher::FileEventKind;
    use crate::features::instance::{Instance, InstanceBuilder, InstanceError};
    use crate::features::minecraft::ModLoader;

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

    struct FixedInstance(Option<Instance>);

    #[async_trait]
    impl GetInstanceUseCasePort for FixedInstance {
        async fn execute(&self, instance_id: String) -> Result<Instance, InstanceError> {
            self.0
                .clone()
                .ok_or(InstanceError::NotFound { instance_id })
        }
    }

    fn instance(stage: InstanceInstallStage) -> Instance {
        let mut instance = InstanceBuilder::new(
            "abc".to_string(),
            "My Pack".to_string(),
            "1.20.1".to_string(),
            ModLoader::Vanilla,
        )
        .build();
        instance.install_stage = stage;
        instance
    }

    fn handler(
        instance: Option<Instance>,
    ) -> (InstanceEventHandler, mpsc::UnboundedReceiver<Event>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let handler = InstanceEventHandler::new(
            Arc::new(ChannelEmitter(tx)),
            Arc::new(FixedInstance(instance)),
        );
        (handler, rx)
    }

    fn modified(path: &str) -> FileEvent {
        FileEvent {
            kind: FileEventKind::Modify,
            path: PathBuf::from(path),
        }
    }

    async fn next_event(rx: &mut mpsc::UnboundedReceiver<Event>) -> Option<Event> {
        tokio::time::timeout(Duration::from_millis(300), rx.recv())
            .await
            .ok()
            .flatten()
    }

    #[tokio::test]
    async fn emits_one_synced_event_per_instance_in_a_batch() {
        let (handler, mut rx) = handler(None);

        handler
            .handle_events(Ok(vec![
                modified("/data/instances/abc/mods/a.jar"),
                modified("/data/instances/abc/mods/b.jar"),
                modified("/data/instances/xyz/mods/c.jar"),
            ]))
            .await
            .unwrap();

        let mut ids = Vec::new();
        while let Some(Event::Instance(event)) = next_event(&mut rx).await {
            assert!(matches!(event.event, InstanceEventType::Synced));
            ids.push(event.instance_id);
        }
        ids.sort();
        assert_eq!(ids, vec!["abc".to_string(), "xyz".to_string()]);
    }

    #[tokio::test]
    async fn ignores_paths_outside_instances_folder() {
        let (handler, mut rx) = handler(None);

        handler
            .handle_events(Ok(vec![modified("/data/other/abc/file.txt")]))
            .await
            .unwrap();

        assert!(next_event(&mut rx).await.is_none());
    }

    #[tokio::test]
    async fn crash_report_of_installed_instance_emits_warning() {
        let (handler, mut rx) = handler(Some(instance(InstanceInstallStage::Installed)));

        handler
            .handle_events(Ok(vec![modified(
                "/data/instances/abc/crash-reports/crash-1.txt",
            )]))
            .await
            .unwrap();

        let Some(Event::Warning(warning)) = next_event(&mut rx).await else {
            panic!("expected warning");
        };
        assert_eq!(
            warning.message,
            "Instance My Pack has crashed! Visit the logs page to see a crash report."
        );
        assert!(next_event(&mut rx).await.is_none());
    }

    #[tokio::test]
    async fn crash_report_of_not_installed_instance_is_silent() {
        let (handler, mut rx) = handler(Some(instance(InstanceInstallStage::NotInstalled)));

        handler
            .handle_events(Ok(vec![modified(
                "/data/instances/abc/crash-reports/crash-1.txt",
            )]))
            .await
            .unwrap();

        assert!(next_event(&mut rx).await.is_none());
    }

    #[tokio::test]
    async fn non_txt_file_in_crash_reports_is_a_regular_sync() {
        let (handler, mut rx) = handler(Some(instance(InstanceInstallStage::Installed)));

        handler
            .handle_events(Ok(vec![modified(
                "/data/instances/abc/crash-reports/readme.md",
            )]))
            .await
            .unwrap();

        assert!(matches!(
            next_event(&mut rx).await,
            Some(Event::Instance(_))
        ));
    }

    #[tokio::test]
    async fn watcher_error_is_swallowed() {
        let (handler, mut rx) = handler(None);

        let result = handler
            .handle_events(Err(FileWatcherError::WatchNotFound))
            .await;

        assert!(result.is_ok());
        assert!(next_event(&mut rx).await.is_none());
    }
}
