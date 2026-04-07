use std::sync::Arc;

use crate::features::{
    events::{ProgressEventType, ProgressService, ProgressServiceExt},
    instance::{Instance, InstanceError, InstanceInstallStage, InstanceStorage},
    java::{JavaInstallationService, JavaStorage, JreProvider},
    minecraft::{
        app::{InstallMinecraftParams, InstallMinecraftUseCase},
        MetadataStorage, MinecraftDownloader,
    },
    settings::LocationInfo,
};

pub struct InstallInstanceUseCase<
    IS: InstanceStorage,
    MS: MetadataStorage,
    MD: MinecraftDownloader,
    PS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    JP: JreProvider,
> {
    instance_storage: Arc<IS>,
    install_minecraft_use_case: Arc<InstallMinecraftUseCase<MS, MD, PS, JIS, JS, JP>>,
    progress_service: Arc<PS>,
    location_info: Arc<LocationInfo>,
}

impl<
        IS: InstanceStorage,
        MS: MetadataStorage,
        MD: MinecraftDownloader,
        PS: ProgressService,
        JIS: JavaInstallationService,
        JS: JavaStorage,
        JP: JreProvider,
    > InstallInstanceUseCase<IS, MS, MD, PS, JIS, JS, JP>
{
    pub fn new(
        instance_storage: Arc<IS>,
        install_minecraft_use_case: Arc<InstallMinecraftUseCase<MS, MD, PS, JIS, JS, JP>>,
        progress_service: Arc<PS>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            instance_storage,
            install_minecraft_use_case,
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
            instance.name,
            instance.game_version,
            instance.loader,
            instance.loader_version
        );

        instance.install_stage = InstanceInstallStage::Installed;
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
                    instance_id: instance.id.clone(),
                    instance_name: instance.name.clone(),
                },
                100.0,
                "Downloading Minecraft".to_string(),
            )
            .await;

        log::info!(
            "Installing instance: \"{}\" (minecraft: \"{}\", modloader: \"{:?}\" \"{:?}\")",
            instance.name,
            instance.game_version,
            instance.loader,
            instance.loader_version
        );

        let result = self
            .install_minecraft_use_case
            .execute(
                InstallMinecraftParams {
                    game_version: instance.game_version.clone(),
                    loader: instance.loader,
                    loader_version: instance.loader_version.clone(),
                    install_dir,
                    java_path: instance.java_path.clone(),
                },
                loading_bar.as_ref(),
                force,
            )
            .await;

        match result {
            Ok(_) => self.handle_success_installation(&mut instance).await,
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
