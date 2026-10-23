use std::sync::Arc;

use async_trait::async_trait;

use crate::features::{
    events::{ProgressBarId, ProgressEventType, ProgressService, ProgressServiceExt},
    instance::{
        Instance, InstanceError, InstanceId, InstanceInstallService, InstanceInstallStage,
        InstanceStorage, InstanceStorageExt,
    },
    minecraft::{InstallMinecraftParams, MinecraftInstallService},
    settings::LocationInfo,
};

pub struct InstallInstanceUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
    install_minecraft_service: Arc<dyn MinecraftInstallService>,
    progress_service: Arc<dyn ProgressService>,
    location_info: Arc<LocationInfo>,
}

impl InstallInstanceUseCase {
    pub fn new(
        instance_storage: Arc<dyn InstanceStorage>,
        install_minecraft_service: Arc<dyn MinecraftInstallService>,
        progress_service: Arc<dyn ProgressService>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            instance_storage,
            install_minecraft_service,
            progress_service,
            location_info,
        }
    }

    async fn handle_success_installation(
        &self,
        instance: &Instance,
        pack_pending: bool,
    ) -> Result<(), InstanceError> {
        log::info!(
            "Installed instance: \"{}\" (minecraft: \"{}\", modloader: \"{:?}\" \"{:?}\")",
            instance.name().to_owned(),
            instance.game_version,
            instance.loader,
            instance.loader_version
        );

        // An instance created for a pack isn't ready until its content is deployed: installing
        // its game keeps it `PackInstalling`, and whoever deploys the pack moves it on. Only that
        // stage is kept — an instance merely *from* a pack (`pack_info`) reinstalled for a health
        // check or a game change is as ready as before.
        let stage = if pack_pending {
            InstanceInstallStage::PackInstalling
        } else {
            InstanceInstallStage::Installed
        };
        self.write_stage(instance.id(), stage).await
    }

    async fn handle_failed_installation(
        &self,
        instance: &Instance,
        previous_stage: InstanceInstallStage,
    ) -> Result<(), InstanceError> {
        // `Installing` was written before the attempt, so a failed reinstall of a working
        // instance must restore `Installed`, and a pack's instance stays `PackInstalling`.
        let stage = match previous_stage {
            InstanceInstallStage::Installed => InstanceInstallStage::Installed,
            InstanceInstallStage::PackInstalling => InstanceInstallStage::PackInstalling,
            _ => InstanceInstallStage::NotInstalled,
        };
        self.write_stage(instance.id(), stage).await
    }

    /// Writes only the stage: the install runs for minutes, and the instance may have been
    /// edited meanwhile.
    async fn write_stage(
        &self,
        id: &InstanceId,
        stage: InstanceInstallStage,
    ) -> Result<(), InstanceError> {
        self.instance_storage
            .upsert_with(id, move |instance| {
                instance.install_stage = stage;
                Ok(())
            })
            .await
    }

    pub async fn execute(
        &self,
        instance_id: InstanceId,
        force: bool,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<(), InstanceError> {
        let instance = self.instance_storage.get(&instance_id).await?;
        let previous_stage = instance.install_stage;
        let pack_pending = previous_stage == InstanceInstallStage::PackInstalling;

        self.write_stage(&instance_id, InstanceInstallStage::Installing)
            .await?;

        let install_dir = self.location_info.instance_dir(&instance_id);

        // Under the bar of a pack being installed into the instance, if any. The shared
        // downloads the game waits on are mirrored under this one.
        let loading_bar = self
            .progress_service
            .init_progress_under_safe(
                parent_bar,
                ProgressEventType::MinecraftDownload {
                    instance_id,
                    instance_name: instance.name().to_owned(),
                },
                100.0,
                "Downloading Minecraft".to_string(),
            )
            .await;

        log::info!(
            "Installing instance: \"{}\" (minecraft: \"{}\", modloader: \"{:?}\" \"{:?}\")",
            instance.name().to_owned(),
            instance.game_version,
            instance.loader,
            instance.loader_version
        );

        let result = self
            .install_minecraft_service
            .execute(
                InstallMinecraftParams {
                    game_version: instance.game_version.clone(),
                    loader: instance.loader,
                    loader_version: instance.loader_version.clone(),
                    install_dir,
                    java_path: if instance.java_path.data.is_empty() {
                        None
                    } else {
                        Some(instance.java_path.data.clone())
                    },
                },
                loading_bar.as_ref(),
                force,
            )
            .await;

        match result {
            Ok(()) => {
                self.handle_success_installation(&instance, pack_pending)
                    .await
            }
            Err(_) => {
                self.handle_failed_installation(&instance, previous_stage)
                    .await
            }
        }?;

        if let Some(loading_bar) = loading_bar {
            match &result {
                Ok(()) => {
                    self.progress_service
                        .emit_progress_safe(
                            &loading_bar,
                            1.000_000_000_01,
                            Some("Finished installing"),
                        )
                        .await;
                }
                Err(err) => loading_bar.fail(err.to_string()).await,
            }
        }

        Ok(result?)
    }
}

#[async_trait]
impl InstanceInstallService for InstallInstanceUseCase {
    async fn execute(
        &self,
        instance_id: InstanceId,
        force: bool,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<(), InstanceError> {
        self.execute(instance_id, force, parent_bar).await
    }
}
