use std::sync::Arc;

use async_trait::async_trait;

use crate::features::{
    file_watcher::FileWatcher,
    instance::{ContentType, InstanceError, InstanceWatcherService},
    settings::LocationInfo,
};

pub struct InstanceWatcherServiceImpl<FW: FileWatcher> {
    file_watcher: Arc<FW>,
    location_info: Arc<LocationInfo>,
}

impl<FW: FileWatcher> InstanceWatcherServiceImpl<FW> {
    pub fn new(file_watcher: Arc<FW>, location_info: Arc<LocationInfo>) -> Self {
        Self {
            file_watcher,
            location_info,
        }
    }
}

#[async_trait]
impl<FW: FileWatcher> InstanceWatcherService for InstanceWatcherServiceImpl<FW> {
    async fn watch_instances(&self) -> Result<(), InstanceError> {
        if let Ok(instances_dir) = std::fs::read_dir(self.location_info.instances_dir()) {
            for instance_dir in instances_dir {
                if let Ok(file_name) = instance_dir.map(|x| x.file_name()) {
                    if let Some(file_name) = file_name.to_str() {
                        if file_name.starts_with(".DS_Store") {
                            continue;
                        };

                        let _ = self.watch_instance(file_name).await;
                    }
                }
            }
        }

        Ok(())
    }

    async fn watch_instance(&self, instance_id: &str) -> Result<(), InstanceError> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        if instance_dir.exists() && instance_dir.is_dir() {
            let folders = ContentType::iterator()
                .map(|x| x.get_folder())
                .chain(["crash-reports"]);

            for folder in folders {
                let path = instance_dir.join(folder);

                if !path.exists() && !path.is_symlink() {
                    if let Err(e) = tokio::fs::create_dir_all(&path).await {
                        tracing::error!("Failed to create directory for watcher {path:?}: {e}");
                        return Ok(());
                    }
                }

                if let Err(e) = self.file_watcher.watch(&path).await {
                    tracing::error!("Failed to watch directory for watcher {path:?}: {e}");
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    async fn unwatch_instance(&self, instance_id: &str) -> Result<(), InstanceError> {
        let path = self.location_info.instance_dir(instance_id);
        if let Err(e) = self.file_watcher.unwatch(&path).await {
            tracing::error!("Failed to unwatch directory for watcher {path:?}: {e}");
        }
        Ok(())
    }
}
