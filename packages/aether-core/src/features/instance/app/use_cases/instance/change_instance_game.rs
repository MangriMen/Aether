use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use log::{error, info};

use crate::features::{
    events::ProgressBarId,
    instance::{
        InstanceError, InstanceGame, InstanceId, InstanceInstallService, InstanceStorage,
        InstanceStorageExt, app::ports::ChangeInstanceGameUseCasePort,
    },
    minecraft::LoaderVersionService,
};

use super::resolve_loader_version;

/// Moves an existing instance to another game version or loader: what a pack update whose
/// new version runs on another game asks for.
///
/// The new game is written to the instance and installed (the client and the loader) as for a
/// new instance. If installing fails, the instance is put back on its old game, in the stage it
/// was in: the old game is still installed, so the instance runs as before.
pub struct ChangeInstanceGameUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
    loader_version_service: Arc<dyn LoaderVersionService>,
    instance_install_service: Arc<dyn InstanceInstallService>,
}

impl ChangeInstanceGameUseCase {
    pub fn new(
        instance_storage: Arc<dyn InstanceStorage>,
        loader_version_service: Arc<dyn LoaderVersionService>,
        instance_install_service: Arc<dyn InstanceInstallService>,
    ) -> Self {
        Self {
            instance_storage,
            loader_version_service,
            instance_install_service,
        }
    }

    pub async fn execute(
        &self,
        instance_id: InstanceId,
        game: InstanceGame,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<(), InstanceError> {
        let instance = self.instance_storage.get(&instance_id).await?;
        if game.is_run_by(&instance) {
            return Ok(());
        }

        let loader_version = resolve_loader_version(
            self.loader_version_service.as_ref(),
            &game.game_version,
            game.mod_loader,
            game.loader_version,
        )
        .await?;

        info!(
            "Moving instance \"{instance_id}\" from minecraft \"{}\" ({:?} {:?}) to \"{}\" ({:?} {:?})",
            instance.game_version,
            instance.loader,
            instance.loader_version,
            game.game_version,
            game.mod_loader,
            loader_version
        );

        let previous = instance;
        self.instance_storage
            .upsert_with(&instance_id, move |instance| {
                instance.game_version = game.game_version;
                instance.loader = game.mod_loader;
                instance.loader_version = loader_version;
                instance.modified = Utc::now();
                Ok(())
            })
            .await?;

        let result = self
            .instance_install_service
            .execute(instance_id.clone(), false, parent_bar)
            .await;

        if let Err(err) = &result {
            info!("Failed to install the new game of \"{instance_id}\": {err}. Rolling back");

            let restored = self
                .instance_storage
                .upsert_with(&instance_id, move |instance| {
                    instance.game_version = previous.game_version;
                    instance.loader = previous.loader;
                    instance.loader_version = previous.loader_version;
                    instance.install_stage = previous.install_stage;
                    instance.modified = previous.modified;
                    Ok(())
                })
                .await;
            if let Err(restore_err) = restored {
                error!("Failed to restore the game of \"{instance_id}\": {restore_err}");
            }
        }

        result
    }
}

#[async_trait]
impl ChangeInstanceGameUseCasePort for ChangeInstanceGameUseCase {
    async fn execute(
        &self,
        instance_id: InstanceId,
        game: InstanceGame,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<(), InstanceError> {
        self.execute(instance_id, game, parent_bar).await
    }
}
