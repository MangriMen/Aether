use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use log::{error, info};
use tokio::fs;

use crate::features::{
    events::{EventEmitterExt, SharedEventEmitter, WarningEvent},
    instance::{
        Instance, InstanceBuilder, InstanceError, InstanceInstallService, InstanceStorage,
        InstanceWatcherService, PackInfo, app::ports::CreateInstanceUseCasePort,
    },
    minecraft::{
        LoaderVersionPreference, LoaderVersionService, MinecraftApplicationError, ModLoader,
    },
    settings::LocationInfo,
};

#[derive(Debug)]
pub struct NewInstance {
    pub name: String,
    pub game_version: String,
    pub mod_loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,
    pub icon_path: Option<String>,
    pub skip_install_instance: Option<bool>,
    pub pack_info: Option<PackInfo>,
}

pub struct CreateInstanceUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
    loader_version_service: Arc<dyn LoaderVersionService>,
    instance_install_service: Arc<dyn InstanceInstallService>,
    location_info: Arc<LocationInfo>,
    event_emitter: SharedEventEmitter,
    instance_watcher_service: Arc<dyn InstanceWatcherService>,
}

impl CreateInstanceUseCase {
    pub fn new(
        instance_storage: Arc<dyn InstanceStorage>,
        loader_version_service: Arc<dyn LoaderVersionService>,
        instance_install_service: Arc<dyn InstanceInstallService>,
        location_info: Arc<LocationInfo>,
        event_emitter: SharedEventEmitter,
        instance_watcher_service: Arc<dyn InstanceWatcherService>,
    ) -> Self {
        Self {
            instance_storage,
            loader_version_service,
            instance_install_service,
            location_info,
            event_emitter,
            instance_watcher_service,
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
                .execute(instance.id().to_owned(), false)
                .await?;
        }

        Ok(instance.id().to_owned())
    }

    pub async fn execute(&self, new_instance: NewInstance) -> Result<String, InstanceError> {
        let NewInstance {
            name,
            game_version,
            mod_loader,
            loader_version,
            icon_path,
            skip_install_instance,
            pack_info,
        } = new_instance;

        let base_sanitized_name = sanitize_instance_name(&name);
        let instances_root = self.location_info.instances_dir();

        let (sanitized_name, instance_dir) =
            create_unique_dir(&instances_root, &base_sanitized_name).await;

        info!(
            "Creating instance \"{}\" at path \"{}\"",
            &name,
            &instance_dir.display()
        );

        // Check that loader version is valid
        let loader_version = if mod_loader != ModLoader::Vanilla && loader_version.is_some() {
            self.loader_version_service
                .resolve(&game_version, &mod_loader, loader_version.as_ref())
                .await
                .map_err(MinecraftApplicationError::Domain)?;

            loader_version
        } else if mod_loader != ModLoader::Vanilla && loader_version.is_none() {
            self.loader_version_service
                .try_get_default(&game_version, &mod_loader)
                .await
                .map_err(MinecraftApplicationError::Domain)?
        } else {
            None
        };

        let instance = build_instance(
            &name,
            &sanitized_name,
            &game_version,
            mod_loader,
            loader_version.as_ref(),
            icon_path.as_ref(),
            pack_info.as_ref(),
        );

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
            if let Err(rmdir_err) = fs::remove_dir_all(&instance_dir).await {
                error!("Failed to remove instance directory during rollback: {rmdir_err}");
            }
        }

        if result.is_ok() {
            info!(
                "Instance \"{}\" created successfully at path \"{}\"",
                instance.name(),
                &instance_dir.display()
            );
        }

        result
    }
}

fn build_instance(
    name: &str,
    sanitized_name: &str,
    game_version: &str,
    mod_loader: ModLoader,
    loader_version: Option<&LoaderVersionPreference>,
    icon_path: Option<&String>,
    pack_info: Option<&PackInfo>,
) -> Instance {
    InstanceBuilder::new(
        sanitized_name.to_owned(),
        name.to_owned(),
        game_version.to_owned(),
        mod_loader,
    )
    .with_loader_version(loader_version.cloned())
    .with_icon(icon_path.cloned())
    .with_pack_info(pack_info.cloned())
    .build()
}

/// Atomically create a unique instance directory by attempting `create_dir`
/// and retrying with an incrementing suffix on `AlreadyExists`.
async fn create_unique_dir(base_dir: &Path, base_name: &str) -> (String, PathBuf) {
    let mut counter = 0u32;
    loop {
        let name = if counter == 0 {
            base_name.to_owned()
        } else {
            format!("{base_name}-{counter}")
        };
        let path = base_dir.join(&name);
        match fs::create_dir(&path).await {
            Ok(()) => return (name, path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                counter += 1;
            }
            Err(e) => {
                panic!(
                    "Failed to create instance directory '{}': {e}",
                    path.display()
                );
            }
        }
    }
}

pub fn sanitize_instance_name(name: &str) -> String {
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
}
