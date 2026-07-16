use std::sync::Arc;

use async_trait::async_trait;

use crate::features::{
    events::{ProgressEventType, ProgressService, ProgressServiceExt},
    instance::{
        Instance, InstanceError, InstanceInstallService, InstanceInstallStage, InstanceStorage,
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
        instance: &mut Instance,
    ) -> Result<(), InstanceError> {
        log::info!(
            "Installed instance: \"{}\" (minecraft: \"{}\", modloader: \"{:?}\" \"{:?}\")",
            instance.name().to_owned(),
            instance.game_version,
            instance.loader,
            instance.loader_version
        );

        if instance.pack_info.is_some() {
            instance.install_stage = InstanceInstallStage::PackInstalling;
        } else {
            instance.install_stage = InstanceInstallStage::Installed;
        }

        self.instance_storage.upsert(instance).await?;
        Ok(())
    }

    async fn handle_failed_installation(
        &self,
        instance: &mut Instance,
    ) -> Result<(), InstanceError> {
        if instance.install_stage != InstanceInstallStage::Installed {
            instance.install_stage = InstanceInstallStage::NotInstalled;
            self.instance_storage.upsert(instance).await?;
        }
        Ok(())
    }

    pub async fn execute(&self, instance_id: String, force: bool) -> Result<(), InstanceError> {
        let mut instance = self.instance_storage.get(&instance_id).await?;

        instance.install_stage = InstanceInstallStage::Installing;
        self.instance_storage.upsert(&instance).await?;

        let install_dir = self.location_info.instance_dir(&instance_id);

        let loading_bar = self
            .progress_service
            .init_progress_safe(
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
            Ok(()) => self.handle_success_installation(&mut instance).await,
            Err(_) => self.handle_failed_installation(&mut instance).await,
        }?;

        if let Some(loading_bar) = loading_bar {
            self.progress_service
                .emit_progress_safe(&loading_bar, 1.000_000_000_01, Some("Finished installing"))
                .await;
        }

        Ok(result?)
    }
}

#[async_trait]
impl InstanceInstallService for InstallInstanceUseCase {
    async fn execute(&self, instance_id: String, force: bool) -> Result<(), InstanceError> {
        self.execute(instance_id, force).await
    }
}
