use std::sync::Arc;

use async_trait::async_trait;

use crate::features::{
    events::{ProgressBarId, ProgressEventType, ProgressService, ProgressServiceExt},
    instance::{
        Instance, InstanceError, InstanceId, InstanceInstallService, InstanceInstallStage,
        InstanceStorage, InstanceStorageExt,
    },
    minecraft::{
        InstallMinecraftParams, LoaderVersionId, LoaderVersionPreference, LoaderVersionService,
        MinecraftInstallService, ModLoader,
    },
    settings::LocationInfo,
};

pub struct InstallInstanceUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
    loader_version_service: Arc<dyn LoaderVersionService>,
    install_minecraft_service: Arc<dyn MinecraftInstallService>,
    progress_service: Arc<dyn ProgressService>,
    location_info: Arc<LocationInfo>,
}

impl InstallInstanceUseCase {
    pub fn new(
        instance_storage: Arc<dyn InstanceStorage>,
        loader_version_service: Arc<dyn LoaderVersionService>,
        install_minecraft_service: Arc<dyn MinecraftInstallService>,
        progress_service: Arc<dyn ProgressService>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            instance_storage,
            loader_version_service,
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
        tracing::info!(
            "Installed instance: \"{}\" (minecraft: \"{}\", modloader: \"{:?}\" \"{:?}\")",
            instance.name(),
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
        self.instance_storage
            .upsert_with(instance.id(), move |instance| {
                instance.install_stage = stage;
                Ok(())
            })
            .await
    }

    async fn handle_failed_installation(
        &self,
        instance_id: &InstanceId,
        previous_stage: InstanceInstallStage,
    ) -> Result<(), InstanceError> {
        // `Installing` was written before the attempt, so a failed reinstall of a working
        // instance must restore `Installed`. A pack's instance stays `PackInstalling`: whoever deploys
        // the pack settles it (discards the new instance, or restores the stage of an update).
        let stage = match previous_stage {
            InstanceInstallStage::Installed | InstanceInstallStage::PackInstalling => {
                previous_stage
            }
            _ => InstanceInstallStage::NotInstalled,
        };
        self.instance_storage
            .upsert_with(instance_id, move |instance| {
                instance.install_stage = stage;
                Ok(())
            })
            .await
    }

    /// The loader version an instance is installed with. An instance that still holds `Stable`
    /// or `Latest` gets the version they resolve to now, stored as exact, so a later reinstall
    /// does not move to a newer one. A loader that cannot be resolved is left as it is: the
    /// install reports why.
    async fn pin_loader_version(&self, instance: &mut Instance) {
        if instance.loader == ModLoader::Vanilla {
            return;
        }
        let Some(preference) = instance.loader_version.clone() else {
            return;
        };
        if matches!(preference, LoaderVersionPreference::Exact(_)) {
            return;
        }

        let resolved = match self
            .loader_version_service
            .resolve(&instance.game_version, &instance.loader, Some(&preference))
            .await
        {
            Ok(Some(resolved)) => resolved,
            Ok(None) => return,
            Err(err) => {
                tracing::warn!(error = %err, "Could not resolve the loader version to pin");
                return;
            }
        };
        let id = match LoaderVersionId::try_from(resolved.id) {
            Ok(id) => id,
            Err(err) => {
                tracing::warn!(error = %err, "The resolved loader version is not a valid id");
                return;
            }
        };
        let exact = LoaderVersionPreference::Exact(id);

        let stored = exact.clone();
        let game_version = instance.game_version.clone();
        let loader = instance.loader;
        match self
            .instance_storage
            .upsert_with(instance.id(), move |it| {
                // Only for the game it was resolved for: an edit that landed meanwhile wins.
                if it.game_version == game_version
                    && it.loader == loader
                    && it.loader_version == Some(preference)
                {
                    it.loader_version = Some(stored);
                }
                Ok(())
            })
            .await
        {
            Ok(()) => instance.loader_version = Some(exact),
            Err(err) => tracing::warn!(error = %err, "Could not store the pinned loader version"),
        }
    }

    pub async fn execute(
        &self,
        instance_id: InstanceId,
        force: bool,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<(), InstanceError> {
        let mut previous_stage = InstanceInstallStage::NotInstalled;
        let mut instance = self
            .instance_storage
            .update(
                &instance_id,
                Box::new(|instance| {
                    previous_stage = instance.install_stage;
                    instance.install_stage = InstanceInstallStage::Installing;
                    Ok(true)
                }),
            )
            .await?;
        let pack_pending = previous_stage == InstanceInstallStage::PackInstalling;
        self.pin_loader_version(&mut instance).await;

        let install_dir = self.location_info.instance_dir(&instance_id);

        // Under the bar of a pack being installed into the instance, if any. The shared
        // downloads the game waits on are mirrored under this one.
        let loading_bar = self
            .progress_service
            .init_progress_under_safe(
                parent_bar,
                ProgressEventType::MinecraftDownload {
                    instance_id,
                    instance_name: instance.name().to_string(),
                },
                100.0,
                "Downloading Minecraft".to_string(),
            )
            .await;

        tracing::info!(
            "Installing instance: \"{}\" (minecraft: \"{}\", modloader: \"{:?}\" \"{:?}\")",
            instance.name(),
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
                    java_path: instance.java_path_override(),
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
                self.handle_failed_installation(instance.id(), previous_stage)
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
