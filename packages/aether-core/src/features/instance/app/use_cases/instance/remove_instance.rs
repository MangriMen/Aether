use std::sync::Arc;

use async_trait::async_trait;
use tracing::warn;

use crate::features::instance::app::ports::RemoveInstanceUseCasePort;
use crate::features::instance::app::{IconAssetReaper, InstanceFileService};
use crate::features::instance::{
    InstallCancellations, InstanceError, InstanceId, InstanceLocks, InstanceStorage,
    InstanceWatcherService, PackStorage,
};
use crate::features::process::RunningGameService;

/// Removes an instance under its lock ([`InstanceLocks`]): after a plan or an update running in
/// it, so they do not write into a directory that is being deleted.
pub struct RemoveInstanceUseCase {
    storage: Arc<dyn InstanceStorage>,
    watcher_service: Arc<dyn InstanceWatcherService>,
    file_service: Arc<dyn InstanceFileService>,
    pack_storage: Arc<dyn PackStorage>,
    instance_locks: Arc<InstanceLocks>,
    icon_reaper: IconAssetReaper,
    running_games: Arc<dyn RunningGameService>,
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
        running_games: Arc<dyn RunningGameService>,
    ) -> Self {
        Self {
            storage,
            watcher_service,
            file_service,
            pack_storage,
            instance_locks,
            icon_reaper,
            running_games,
            cancellations: Arc::default(),
        }
    }

    /// Looked up by instance id rather than read from the instance, so a broken instance is checked
    /// too. A game only a saved record tells of counts, as for a launch.
    async fn ensure_not_running(&self, instance_id: &InstanceId) -> Result<(), InstanceError> {
        match self.running_games.running_game(instance_id).await? {
            Some(process_id) => Err(InstanceError::InstanceAlreadyRunning {
                instance_id: instance_id.to_string(),
                process_id,
            }),
            None => Ok(()),
        }
    }

    /// Remove under a registry shared with the runs for instances, so that a run going on for
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
        // Before the removal is announced, so a refusal does not call off a legitimate install.
        self.ensure_not_running(&instance_id).await?;
        // Before the lock, which a launch holds through its install. Waited for: the install
        // restores the stage of the instance as it stops, which must come before the removal.
        let _removal = self.cancellations.begin_removal(&instance_id).await;
        let _lock = self.instance_locks.lock(&instance_id).await;
        // A launch may have started the game while the lock was awaited.
        self.ensure_not_running(&instance_id).await?;

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
