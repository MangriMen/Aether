use std::sync::Arc;

use async_trait::async_trait;
use log::{error, info};

use crate::features::{
    events::{EventEmitterExt, SharedEventEmitter, WarningEvent},
    instance::{
        Instance, InstanceBuilder, InstanceError, InstanceFileService, InstanceId,
        InstanceInstallService, InstanceInstallStage, InstanceStorage, InstanceWatcherService,
        NewInstance, app::ports::CreateInstanceUseCasePort,
    },
    minecraft::{
        LoaderVersionPreference, LoaderVersionService, MinecraftApplicationError, ModLoader,
    },
};
use crate::shared::cache::domain::AssetsStorage;

pub struct CreateInstanceUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
    loader_version_service: Arc<dyn LoaderVersionService>,
    instance_install_service: Arc<dyn InstanceInstallService>,
    instance_file_service: Arc<dyn InstanceFileService>,
    event_emitter: SharedEventEmitter,
    instance_watcher_service: Arc<dyn InstanceWatcherService>,
    assets_storage: Arc<dyn AssetsStorage>,
}

impl CreateInstanceUseCase {
    pub fn new(
        instance_storage: Arc<dyn InstanceStorage>,
        loader_version_service: Arc<dyn LoaderVersionService>,
        instance_install_service: Arc<dyn InstanceInstallService>,
        instance_file_service: Arc<dyn InstanceFileService>,
        event_emitter: SharedEventEmitter,
        instance_watcher_service: Arc<dyn InstanceWatcherService>,
        assets_storage: Arc<dyn AssetsStorage>,
    ) -> Self {
        Self {
            instance_storage,
            loader_version_service,
            instance_install_service,
            instance_file_service,
            event_emitter,
            instance_watcher_service,
            assets_storage,
        }
    }

    /// The icon of a new instance is named by a file; the instance keeps the id of its copy in
    /// the asset store, the only thing the UI can show. A host path kept as it is would read as
    /// a missing asset. An icon that cannot be read is dropped with a warning: it is no reason
    /// to refuse the instance.
    async fn import_icon(&self, icon_path: Option<String>) -> Option<String> {
        let icon_path = icon_path?;

        match self
            .assets_storage
            .import_file(std::path::Path::new(&icon_path))
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
    ) -> Result<String, InstanceError> {
        self.instance_storage.upsert(instance).await?;

        self.instance_watcher_service
            .watch_instance(instance.id())
            .await?;

        if !skip_install_instance.unwrap_or(false) {
            self.instance_install_service
                .execute(instance.id().clone(), false)
                .await?;
        }

        Ok(instance.id().to_string())
    }

    pub async fn execute(&self, new_instance: NewInstance) -> Result<String, InstanceError> {
        self.create(new_instance, InstanceInstallStage::NotInstalled)
            .await
    }

    /// See [`CreateInstanceUseCasePort::execute_for_pack`].
    pub async fn execute_for_pack(
        &self,
        new_instance: NewInstance,
    ) -> Result<String, InstanceError> {
        self.create(new_instance, InstanceInstallStage::PackInstalling)
            .await
    }

    /// `initial_stage` — `PackInstalling` keeps the instance unlaunchable through the install of
    /// its game, which preserves that stage.
    async fn create(
        &self,
        new_instance: NewInstance,
        initial_stage: InstanceInstallStage,
    ) -> Result<String, InstanceError> {
        let NewInstance {
            name,
            game_version,
            mod_loader,
            loader_version,
            icon_path,
            skip_install_instance,
            pack_info,
        } = new_instance;

        let icon = self.import_icon(icon_path).await;

        let base_sanitized_name = sanitize_instance_name(&name);

        let (sanitized_name, instance_dir) = self
            .instance_file_service
            .create_unique_instance_dir(&base_sanitized_name)
            .await?;

        info!(
            "Creating instance \"{}\" at path \"{}\"",
            name,
            instance_dir.display()
        );

        let loader_version = resolve_loader_version(
            self.loader_version_service.as_ref(),
            &game_version,
            mod_loader,
            loader_version,
        )
        .await?;

        let mut instance = InstanceBuilder::new(
            InstanceId::try_from(sanitized_name)?,
            name.clone(),
            game_version.clone(),
            mod_loader,
        )
        .with_loader_version(loader_version)
        .with_icon(icon)
        .with_pack_info(pack_info)
        .build();
        instance.install_stage = initial_stage;

        let result = self.setup_instance(&instance, skip_install_instance).await;

        // Full rollback on any failure: unwatch, remove from storage, delete directory
        if let Err(err) = &result {
            info!(
                "Failed to create instance \"{}\". Rolling back",
                instance.name()
            );

            self.event_emitter
                .emit_safe(WarningEvent {
                    message: format!("Error creating instance {err}"),
                })
                .await;

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
/// checked to exist, or the default one when it is `None`; nothing for vanilla.
pub(crate) async fn resolve_loader_version(
    loader_version_service: &dyn LoaderVersionService,
    game_version: &str,
    mod_loader: ModLoader,
    loader_version: Option<LoaderVersionPreference>,
) -> Result<Option<LoaderVersionPreference>, InstanceError> {
    if mod_loader == ModLoader::Vanilla {
        return Ok(None);
    }

    match loader_version {
        Some(loader_version) => {
            loader_version_service
                .resolve(game_version, &mod_loader, Some(&loader_version))
                .await
                .map_err(MinecraftApplicationError::Domain)?;

            Ok(Some(loader_version))
        }
        None => Ok(loader_version_service
            .try_get_default(game_version, &mod_loader)
            .await
            .map_err(MinecraftApplicationError::Domain)?),
    }
}

pub(crate) fn sanitize_instance_name(name: &str) -> String {
    name.replace(
        ['/', '\\', '?', '*', ':', '\'', '\"', '|', '<', '>', '!'],
        "_",
    )
}

#[async_trait]
impl CreateInstanceUseCasePort for CreateInstanceUseCase {
    async fn execute(&self, new_instance: NewInstance) -> Result<String, InstanceError> {
        self.execute(new_instance).await
    }

    async fn execute_for_pack(&self, new_instance: NewInstance) -> Result<String, InstanceError> {
        self.execute_for_pack(new_instance).await
    }
}
