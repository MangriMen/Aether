use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::app::ports::RemoveInstanceUseCasePort;
use crate::features::instance::app::{IconAssetReaper, InstanceFileService};
use crate::features::instance::{
    InstanceError, InstanceId, InstanceLocks, InstanceStorage, InstanceWatcherService, PackStorage,
};

/// Removes an instance under its lock ([`InstanceLocks`]): after a plan or an update running in
/// it, so they do not write into a directory that is being deleted.
pub struct RemoveInstanceUseCase {
    storage: Arc<dyn InstanceStorage>,
    watcher_service: Arc<dyn InstanceWatcherService>,
    file_service: Arc<dyn InstanceFileService>,
    pack_storage: Arc<dyn PackStorage>,
    instance_locks: Arc<InstanceLocks>,
    icon_reaper: Option<IconAssetReaper>,
}

impl RemoveInstanceUseCase {
    pub fn new(
        storage: Arc<dyn InstanceStorage>,
        watcher_service: Arc<dyn InstanceWatcherService>,
        file_service: Arc<dyn InstanceFileService>,
        pack_storage: Arc<dyn PackStorage>,
        instance_locks: Arc<InstanceLocks>,
    ) -> Self {
        Self {
            storage,
            watcher_service,
            file_service,
            pack_storage,
            instance_locks,
            icon_reaper: None,
        }
    }

    /// Also deletes the instance's icon asset when no other instance uses it.
    #[must_use]
    pub fn with_icon_reaper(mut self, icon_reaper: IconAssetReaper) -> Self {
        self.icon_reaper = Some(icon_reaper);
        self
    }
}

#[async_trait]
impl RemoveInstanceUseCasePort for RemoveInstanceUseCase {
    async fn execute(&self, instance_id: InstanceId) -> Result<(), InstanceError> {
        let _lock = self.instance_locks.lock(&instance_id).await;

        let previous_icon = match &self.icon_reaper {
            Some(_) => self
                .storage
                .get(&instance_id)
                .await
                .ok()
                .and_then(|instance| instance.icon_path().map(ToOwned::to_owned)),
            None => None,
        };

        self.watcher_service.unwatch_instance(&instance_id).await?;
        // Files first: if the directory cannot be removed, the records stay and the removal can be
        // retried instead of leaving a listed instance without its pack data.
        self.file_service.remove_instance_dir(&instance_id).await?;
        self.pack_storage
            .remove_all_for_instance(&instance_id)
            .await?;
        self.storage.remove(&instance_id).await?;
        if let (Some(reaper), Some(icon)) = (&self.icon_reaper, previous_icon) {
            reaper.release(&icon).await;
        }
        Ok(())
    }
}
