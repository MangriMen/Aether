use std::sync::Arc;

use async_trait::async_trait;
use tracing::warn;

use crate::features::instance::app::ports::RemoveInstanceUseCasePort;
use crate::features::instance::app::{IconAssetReaper, InstanceFileService};
use crate::features::instance::{
    InstallCancellations, InstanceError, InstanceId, InstanceLocks, InstanceStorage,
    InstanceWatcherService, PackStorage,
};

/// Removes an instance under its lock ([`InstanceLocks`]): after a plan or an update running in
/// it, so they do not write into a directory that is being deleted.
pub struct RemoveInstanceUseCase {
    storage: Arc<dyn InstanceStorage>,
    watcher_service: Arc<dyn InstanceWatcherService>,
    file_service: Arc<dyn InstanceFileService>,
    pack_storage: Arc<dyn PackStorage>,
    instance_locks: Arc<InstanceLocks>,
    icon_reaper: IconAssetReaper,
    cancellations: Arc<InstallCancellations>,
}

impl RemoveInstanceUseCase {
    pub fn new(
        storage: Arc<dyn InstanceStorage>,
        watcher_service: Arc<dyn InstanceWatcherService>,
        file_service: Arc<dyn InstanceFileService>,
        pack_storage: Arc<dyn PackStorage>,
        instance_locks: Arc<InstanceLocks>,
        icon_reaper: IconAssetReaper,
    ) -> Self {
        Self {
            storage,
            watcher_service,
            file_service,
            pack_storage,
            instance_locks,
            icon_reaper,
            cancellations: Arc::default(),
        }
    }

    /// Remove under a registry shared with the install of games, so that an install running for
    /// the instance is called off.
    #[must_use]
    pub fn with_cancellations(mut self, cancellations: Arc<InstallCancellations>) -> Self {
        self.cancellations = cancellations;
        self
    }
}

#[async_trait]
impl RemoveInstanceUseCasePort for RemoveInstanceUseCase {
    async fn execute(&self, instance_id: InstanceId) -> Result<(), InstanceError> {
        // Before the lock, which a launch holds through its install. Waited for: the install
        // restores the stage of the instance as it stops, which must come before the removal.
        self.cancellations.cancel_and_wait(&instance_id).await;
        let _lock = self.instance_locks.lock(&instance_id).await;

        let previous_icon = match self.storage.get(&instance_id).await {
            Ok(instance) => instance.icon_path().map(ToOwned::to_owned),
            Err(err) => {
                warn!("Cannot read the icon of instance {instance_id}, its asset may stay: {err}");
                None
            }
        };

        self.watcher_service.unwatch_instance(&instance_id).await?;
        // Files first: if the directory cannot be removed, the records stay and the removal can be
        // retried instead of leaving a listed instance without its pack data.
        if let Err(err) = self.file_service.remove_instance_dir(&instance_id).await {
            // The directory is untouched (see `remove_instance_dir`), so it is still a live
            // instance and must keep being watched.
            if let Err(watch_err) = self.watcher_service.watch_instance(&instance_id).await {
                warn!(
                    "Failed to watch instance {instance_id} again after a failed removal: {watch_err}"
                );
            }
            return Err(err);
        }
        self.pack_storage
            .remove_all_for_instance(&instance_id)
            .await?;
        self.storage.remove(&instance_id).await?;
        if let Some(icon) = previous_icon {
            self.icon_reaper.release(&icon).await;
        }
        Ok(())
    }
}
