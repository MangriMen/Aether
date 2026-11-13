use std::sync::Arc;

use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use crate::features::{
    events::{ProgressBarId, ProgressEventType, ProgressService, ProgressServiceExt},
    instance::{
        InstallCancellations, Instance, InstanceError, InstanceId, InstanceInstallService,
        InstanceInstallStage, InstanceStorage, InstanceStorageExt,
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
    cancellations: Arc<InstallCancellations>,
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
            cancellations: Arc::default(),
        }
    }

    /// Install under a registry shared with whoever removes instances, which calls off the
    /// install of the instance it removes.
    #[must_use]
    pub fn with_cancellations(mut self, cancellations: Arc<InstallCancellations>) -> Self {
        self.cancellations = cancellations;
        self
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

    /// Write the stage the install ended in. What the install ended in comes first; a stage that
    /// could not be written is logged.
    async fn settle_stage(
        &self,
        instance: &Instance,
        result: Result<(), InstanceError>,
        pack_pending: bool,
        previous_stage: InstanceInstallStage,
    ) -> Result<(), InstanceError> {
        let settled = match result {
            Ok(()) => {
                self.handle_success_installation(instance, pack_pending)
                    .await
            }
            Err(_) => {
                self.handle_failed_installation(instance.id(), previous_stage)
                    .await
            }
        };
        match (result, settled) {
            (Ok(()), Err(err)) => Err(err),
            (result, Err(err)) => {
                tracing::warn!(error = %err, "Could not settle the install stage of \"{}\"", instance.id());
                result
            }
            (result, Ok(())) => result,
        }
    }

    pub async fn execute(
        &self,
        instance_id: InstanceId,
        force: bool,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<(), InstanceError> {
        // From the start: a removal that comes while the loader version is still resolved
        // calls this install off too.
        let removal = self.cancellations.register(&instance_id);
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
        let event_type = ProgressEventType::MinecraftDownload {
            instance_id,
            instance_name: instance.name().to_string(),
        };
        // A bar of its own can be called off by the user; under a pack's bar the pack's is the
        // one that is, and its install of the game ends in a point of no return of its own.
        let (loading_bar, cancel) = match parent_bar {
            Some(parent) => (
                self.progress_service
                    .init_child_progress_safe(
                        parent.id,
                        event_type,
                        100.0,
                        "Downloading Minecraft".to_string(),
                    )
                    .await,
                CancellationToken::new(),
            ),
            None => {
                self.progress_service
                    .init_cancellable_progress_safe(
                        event_type,
                        100.0,
                        "Downloading Minecraft".to_string(),
                    )
                    .await
            }
        };

        tracing::info!(
            "Installing instance: \"{}\" (minecraft: \"{}\", modloader: \"{:?}\" \"{:?}\")",
            instance.name(),
            instance.game_version,
            instance.loader,
            instance.loader_version
        );

        let params = InstallMinecraftParams {
            game_version: instance.game_version.clone(),
            loader: instance.loader,
            loader_version: instance.loader_version.clone(),
            install_dir,
            java_path: instance.java_path_override(),
        };
        // Dropped halfway on a call-off: what is downloaded stays, the files are written whole
        // and the next install takes up where this one stopped. Called off by the button of the
        // bar, or by the removal of the instance.
        let result = tokio::select! {
            () = cancel.cancelled() => Err(InstanceError::Cancelled),
            () = removal.token.cancelled() => Err(InstanceError::Cancelled),
            result = self.install_minecraft_service.execute(params, loading_bar.as_ref(), force) => {
                result.map_err(InstanceError::from)
            }
        };

        if let Some(bar) = loading_bar.as_ref().filter(|_| result.is_ok()) {
            // Done: too late to call it off.
            self.progress_service.close_cancellation_safe(bar).await;
        }

        let result = self
            .settle_stage(&instance, result, pack_pending, previous_stage)
            .await;
        drop(removal);

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
                Err(InstanceError::Cancelled) => loading_bar.cancel().await,
                Err(err) => loading_bar.fail(err.clone()).await,
            }
        }

        result
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
