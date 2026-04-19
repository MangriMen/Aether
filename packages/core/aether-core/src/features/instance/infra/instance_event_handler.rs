use std::path::Path;

use async_trait::async_trait;

use crate::features::{
    events::{EventEmitterExt, InstanceEvent, InstanceEventType, SharedEventEmitter, WarningEvent},
    file_watcher::{FileEvent, FileEventHandler, FileWatcherError},
    instance::InstanceInstallStage,
    settings::INSTANCES_FOLDER_NAME,
};

pub struct InstanceEventHandler {
    event_emitter: SharedEventEmitter,
}

impl InstanceEventHandler {
    pub fn new(event_emitter: SharedEventEmitter) -> Self {
        Self { event_emitter }
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

    fn crash_task(&self, id: String) {
        tokio::task::spawn({
            let event_emitter = self.event_emitter.clone();
            async move {
                let res = async {
                    let instance = crate::api::instance::get(id).await;

                    if let Ok(instance) = instance {
                        // Don't show warning if profile is not yet installed
                        if instance.install_stage == InstanceInstallStage::Installed {
                            let message = format!(
                            "Profile {} has crashed! Visit the logs page to see a crash report.",
                                instance.name
                            );

                            event_emitter.emit_safe(WarningEvent { message }).await;
                        }
                    }

                    Ok::<(), crate::Error>(())
                }
                .await;

                match res {
                    Ok(()) => {}
                    Err(err) => {
                        tracing::warn!("Unable to send crash report to frontend: {err}");
                    }
                }
            }
        });
    }
}

#[async_trait]
impl FileEventHandler for InstanceEventHandler {
    async fn handle_events(
        &self,
        events: Result<Vec<FileEvent>, FileWatcherError>,
    ) -> Result<(), FileWatcherError> {
        match events {
            Ok(events) => {
                let mut visited_profiles = Vec::new();

                for event in &events {
                    if let Some(instance_path) = Self::extract_instance_path(&event.path) {
                        if Self::is_crash_report(&event.path) {
                            self.crash_task(instance_path.clone());
                        } else if !visited_profiles.contains(&instance_path) {
                            let path = instance_path.clone();

                            tokio::spawn({
                                let event_emitter = self.event_emitter.clone();
                                async move {
                                    let () = event_emitter
                                        .emit_safe(InstanceEvent {
                                            event: InstanceEventType::Synced,
                                            instance_id: path,
                                        })
                                        .await;
                                }
                            });
                            visited_profiles.push(instance_path);
                        }
                    }
                }
            }
            Err(error) => tracing::warn!("Unable to watch file: {error}"),
        }

        Ok(())
    }
}
