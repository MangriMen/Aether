use std::sync::Arc;

use async_trait::async_trait;
use log::{error, info};

use crate::features::{
    events::{EventEmitterExt, ProgressBarId, SharedEventEmitter, WarningEvent},
    instance::{
        IconAssetGuard, IconAssetReaper, Instance, InstanceBuilder, InstanceError,
        InstanceFileService, InstanceId, InstanceInstallService, InstanceInstallStage,
        InstanceStorage, InstanceWatcherService, NewInstance,
        app::ports::CreateInstanceUseCasePort,
    },
    minecraft::{
        GameVersion, LoaderVersionId, LoaderVersionPreference, LoaderVersionService,
        MinecraftApplicationError, ModLoader,
    },
};

pub struct CreateInstanceUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
    loader_version_service: Arc<dyn LoaderVersionService>,
    instance_install_service: Arc<dyn InstanceInstallService>,
    instance_file_service: Arc<dyn InstanceFileService>,
    event_emitter: SharedEventEmitter,
    instance_watcher_service: Arc<dyn InstanceWatcherService>,
    icon_reaper: IconAssetReaper,
}

impl CreateInstanceUseCase {
    pub fn new(
        instance_storage: Arc<dyn InstanceStorage>,
        loader_version_service: Arc<dyn LoaderVersionService>,
        instance_install_service: Arc<dyn InstanceInstallService>,
        instance_file_service: Arc<dyn InstanceFileService>,
        event_emitter: SharedEventEmitter,
        instance_watcher_service: Arc<dyn InstanceWatcherService>,
        icon_reaper: IconAssetReaper,
    ) -> Self {
        Self {
            instance_storage,
            loader_version_service,
            instance_install_service,
            instance_file_service,
            event_emitter,
            instance_watcher_service,
            icon_reaper,
        }
    }

    /// The icon of a new instance is named by a file; the instance keeps the id of its copy in
    /// the asset store, the only thing the UI can show. A host path kept as it is would read as
    /// a missing asset. An icon that cannot be read is dropped with a warning: it is no reason
    /// to refuse the instance.
    async fn import_icon(
        &self,
        guard: &IconAssetGuard<'_>,
        icon_path: Option<String>,
    ) -> Option<String> {
        let icon_path = icon_path?;

        match self
            .icon_reaper
            .import_file(guard, std::path::Path::new(&icon_path))
            .await
        {
            Ok(asset_id) => Some(asset_id),
            Err(err) => {
                error!("Failed to import the instance icon: {err}");
                self.event_emitter
                    .emit_safe(WarningEvent {
                        message: format!("The instance icon was not imported: {err}"),
                    })
                    .await;
                None
            }
        }
    }

    async fn setup_instance(
        &self,
        instance: &Instance,
        skip_install_instance: Option<bool>,
        parent_bar: Option<&ProgressBarId>,
        icon_guard: IconAssetGuard<'_>,
    ) -> Result<InstanceId, InstanceError> {
        let stored = self.instance_storage.upsert(instance).await;
        // The instance holds the icon from now on.
        drop(icon_guard);
        stored?;

        self.instance_watcher_service
            .watch_instance(instance.id())
            .await?;

        if !skip_install_instance.unwrap_or(false) {
            self.instance_install_service
                .execute(instance.id().clone(), false, parent_bar)
                .await?;
        }

        Ok(instance.id().clone())
    }

    pub async fn execute(
        &self,
        new_instance: NewInstance,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<InstanceId, InstanceError> {
        self.create(new_instance, InstanceInstallStage::NotInstalled, parent_bar)
            .await
    }

    /// See [`CreateInstanceUseCasePort::execute_for_pack`].
    pub async fn execute_for_pack(
        &self,
        new_instance: NewInstance,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<InstanceId, InstanceError> {
        self.create(
            new_instance,
            InstanceInstallStage::PackInstalling,
            parent_bar,
        )
        .await
    }

    /// `initial_stage` — `PackInstalling` keeps the instance unlaunchable through the install of
    /// its game, which preserves that stage.
    async fn create(
        &self,
        new_instance: NewInstance,
        initial_stage: InstanceInstallStage,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<InstanceId, InstanceError> {
        let NewInstance {
            name,
            game_version,
            mod_loader,
            loader_version,
            icon_path,
            skip_install_instance,
            pack_info,
        } = new_instance;

        // Parsed before the loader and before anything exists on disk: a name that cannot be an
        // id must fail while there is still nothing to roll back.
        let base_id = InstanceId::from_name(&name)?;

        // Before anything is created on disk: a bad loader must not leave a directory behind.
        let loader_version = resolve_loader_version(
            self.loader_version_service.as_ref(),
            &game_version,
            mod_loader,
            loader_version,
        )
        .await?;

        let icon_guard = self.icon_reaper.hold().await;
        let icon = self.import_icon(&icon_guard, icon_path).await;

        let (instance_id, instance_dir) = match self
            .instance_file_service
            .create_unique_instance_dir(&base_id)
            .await
        {
            Ok(created) => created,
            Err(err) => {
                // No instance will hold the imported icon; releasing needs the guard dropped.
                drop(icon_guard);
                if let Some(icon) = &icon {
                    self.icon_reaper.release(icon).await;
                }
                return Err(err);
            }
        };

        info!(
            "Creating instance \"{}\" at path \"{}\"",
            name,
            instance_dir.display()
        );

        let mut instance =
            InstanceBuilder::new(instance_id, name.clone(), game_version.clone(), mod_loader)
                .with_loader_version(loader_version)
                .with_icon(icon)
                .with_pack_info(pack_info)
                .build();
        instance.install_stage = initial_stage;

        let result = self
            .setup_instance(&instance, skip_install_instance, parent_bar, icon_guard)
            .await;

        // Full rollback on any failure: unwatch, remove from storage, delete directory
        if let Err(err) = &result {
            info!(
                "Failed to create instance \"{}\". Rolling back",
                instance.name()
            );

            // Called off by the user: not something to warn about.
            if !matches!(err, InstanceError::Cancelled) {
                self.event_emitter
                    .emit_safe(WarningEvent {
                        message: format!("Error creating instance {err}"),
                    })
                    .await;
            }

            if let Err(unwatch_err) = self
                .instance_watcher_service
                .unwatch_instance(instance.id())
                .await
            {
                error!("Failed to unwatch instance during rollback: {unwatch_err}");
            }
            if let Err(remove_err) = self.instance_storage.remove(instance.id()).await {
                error!("Failed to remove instance during rollback: {remove_err}");
            }
            if let Err(rmdir_err) = self
                .instance_file_service
                .remove_instance_dir(instance.id())
                .await
            {
                error!("Failed to remove instance directory during rollback: {rmdir_err}");
            }
            if let Some(icon) = instance.icon_path() {
                self.icon_reaper.release(icon).await;
            }
        }

        if result.is_ok() {
            info!(
                "Instance \"{}\" created successfully at path \"{}\"",
                instance.name(),
                instance_dir.display()
            );
        }

        result
    }
}

/// The loader version an instance of `game_version` and `mod_loader` gets: `loader_version`
/// (or the default one when it is `None`) resolved against the manifest and pinned to the
/// concrete version it names, so that `Stable` and `Latest` do not drift on a later install;
/// nothing for vanilla.
pub(crate) async fn resolve_loader_version(
    loader_version_service: &dyn LoaderVersionService,
    game_version: &GameVersion,
    mod_loader: ModLoader,
    loader_version: Option<LoaderVersionPreference>,
) -> Result<Option<LoaderVersionPreference>, InstanceError> {
    if mod_loader == ModLoader::Vanilla {
        return Ok(None);
    }

    let preference = match loader_version {
        Some(preference) => preference,
        None => match loader_version_service
            .try_get_default(game_version, &mod_loader)
            .await
            .map_err(MinecraftApplicationError::Domain)?
        {
            Some(preference) => preference,
            None => return Ok(None),
        },
    };

    let resolved = loader_version_service
        .resolve(game_version, &mod_loader, Some(&preference))
        .await
        .map_err(MinecraftApplicationError::Domain)?;

    pin(preference, resolved.map(|it| it.id)).map(Some)
}

/// `preference` made exact: the id the manifest resolved it to, or `preference` itself when the
/// service named none.
fn pin(
    preference: LoaderVersionPreference,
    resolved_id: Option<String>,
) -> Result<LoaderVersionPreference, InstanceError> {
    Ok(match resolved_id {
        Some(id) => LoaderVersionPreference::Exact(LoaderVersionId::try_from(id)?),
        None => preference,
    })
}

#[async_trait]
impl CreateInstanceUseCasePort for CreateInstanceUseCase {
    async fn execute(
        &self,
        new_instance: NewInstance,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<InstanceId, InstanceError> {
        self.execute(new_instance, parent_bar).await
    }

    async fn execute_for_pack(
        &self,
        new_instance: NewInstance,
        parent_bar: Option<&ProgressBarId>,
    ) -> Result<InstanceId, InstanceError> {
        self.execute_for_pack(new_instance, parent_bar).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_pin_the_resolved_id_when_the_preference_is_stable_or_latest() {
        for preference in [
            LoaderVersionPreference::Stable,
            LoaderVersionPreference::Latest,
        ] {
            assert_eq!(
                pin(preference, Some("0.16.10".to_string())).unwrap(),
                LoaderVersionPreference::Exact(LoaderVersionId::from_static("0.16.10"))
            );
        }
    }

    #[test]
    fn should_keep_the_preference_when_nothing_was_resolved() {
        assert_eq!(
            pin(LoaderVersionPreference::Stable, None).unwrap(),
            LoaderVersionPreference::Stable
        );
    }

    #[test]
    fn should_refuse_a_resolved_id_when_it_is_not_a_valid_loader_version() {
        let err = pin(LoaderVersionPreference::Stable, Some("1 2".to_string())).unwrap_err();

        assert!(matches!(err, InstanceError::InvalidLoaderVersion { .. }));
    }
}
