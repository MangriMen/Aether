use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        auth::Credential,
        instance::{
            Instance, InstanceError, InstanceId, InstanceInstallService, InstanceInstallStage,
            InstanceLaunchService, InstanceLocks, InstanceStorage, InstanceStorageExt, LockRefused,
        },
        minecraft::{
            GetMinecraftLaunchCommandParams, LaunchSettings, MinecraftHealthParams,
            MinecraftHealthService, MinecraftLaunchCommandService,
        },
        plugins::{PluginInternalEvent, PluginRegistry, PluginState},
        process::{MinecraftProcessMetadata, ProcessStartService, ProcessStorage},
        settings::{DefaultInstanceSettings, DefaultInstanceSettingsStorage, LocationInfo},
    },
    shared::{io::domain::IoError, serializable_command::domain::SerializableCommand},
};

pub struct LaunchInstanceUseCase {
    plugin_registry: Arc<PluginRegistry>,
    instance_storage: Arc<dyn InstanceStorage>,
    default_instance_settings_storage: Arc<dyn DefaultInstanceSettingsStorage>,
    location_info: Arc<LocationInfo>,
    process_storage: Arc<dyn ProcessStorage>,
    instance_install_service: Arc<dyn InstanceInstallService>,
    minecraft_health_service: Arc<dyn MinecraftHealthService>,
    minecraft_launch_command_service: Arc<dyn MinecraftLaunchCommandService>,
    process_start_service: Arc<dyn ProcessStartService>,
    instance_locks: Arc<InstanceLocks>,
}

impl LaunchInstanceUseCase {
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    pub fn new(
        plugin_registry: Arc<PluginRegistry>,
        instance_storage: Arc<dyn InstanceStorage>,
        default_instance_settings_storage: Arc<dyn DefaultInstanceSettingsStorage>,
        location_info: Arc<LocationInfo>,
        process_storage: Arc<dyn ProcessStorage>,
        instance_install_service: Arc<dyn InstanceInstallService>,
        minecraft_health_service: Arc<dyn MinecraftHealthService>,
        minecraft_launch_command_service: Arc<dyn MinecraftLaunchCommandService>,
        process_start_service: Arc<dyn ProcessStartService>,
        instance_locks: Arc<InstanceLocks>,
    ) -> Self {
        Self {
            plugin_registry,
            instance_storage,
            default_instance_settings_storage,
            location_info,
            process_storage,
            instance_install_service,
            minecraft_health_service,
            minecraft_launch_command_service,
            process_start_service,
            instance_locks,
        }
    }

    fn resolve_launch_settings(
        instance: &Instance,
        settings: &DefaultInstanceSettings,
    ) -> LaunchSettings {
        LaunchSettings {
            launch_args: instance.launch_args.resolve(settings.launch_args()),
            env_vars: instance.env_vars.resolve(settings.env_vars()),
            memory: instance.memory.resolve(settings.memory()),
            window: instance.window.resolve(settings.window()),
            hooks: instance.hooks.resolve(settings.hooks()),
        }
    }

    async fn notify_plugin_event(
        &self,
        instance: &Instance,
        event: PluginInternalEvent,
        event_name: &str,
    ) {
        // The handle is cloned out of the registry: holding the entry over the call would block
        // a disable or restart of the plugin, which writes it, on the runtime thread. A plugin
        // that is not loaded, or stops while it is called, only misses the event.
        let handle = instance.pack_info.as_ref().and_then(|pack_info| {
            let plugin = self
                .plugin_registry
                .get(&pack_info.provider_id.plugin_id)
                .ok()?;
            match &plugin.state {
                PluginState::Loaded(handle) => Some(handle.clone()),
                _ => None,
            }
        });
        if let Some(pack_info) = &instance.pack_info
            && let Some(handle) = handle
            && let Err(e) = handle
                .run("handle_event", move |plugin| plugin.handle_event(&event))
                .await
                .flatten()
        {
            tracing::warn!(
                "Plugin '{}' failed to handle {}: {}",
                pack_info.provider_id.plugin_id,
                event_name,
                e
            );
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn execute(
        &self,
        instance_id: InstanceId,
        credentials: Credential,
    ) -> Result<MinecraftProcessMetadata, InstanceError> {
        // Held until the game is started: content is not installed or updated under it. The
        // stage below does not tell an install into an `Installed` instance, nor one that starts
        // after it was read.
        let _lock = match self.instance_locks.try_lock(&instance_id) {
            Ok(lock) => lock,
            Err(LockRefused::Held) => {
                return Err(InstanceError::InstanceStillInstalling {
                    instance_id: instance_id.into(),
                });
            }
            Err(LockRefused::Paused) => return Err(InstanceError::CachesBeingCleared),
        };
        let settings = self.default_instance_settings_storage.get().await?;
        let instance = self.instance_storage.get(&instance_id).await?;

        let launch_settings = Self::resolve_launch_settings(&instance, &settings);

        if instance.install_stage == InstanceInstallStage::PackInstalling
            || instance.install_stage == InstanceInstallStage::Installing
        {
            return Err(InstanceError::InstanceStillInstalling {
                instance_id: instance_id.into(),
            });
        }

        // Check if profile has a running profile, and reject running the command if it does
        // Done late so a quick double call doesn't launch two instances
        if let Some(process) = self
            .process_storage
            .list_metadata()
            .await?
            .into_iter()
            .find(|p| *p.instance_id() == instance_id)
        {
            return Err(InstanceError::InstanceAlreadyRunning {
                instance_id: instance_id.to_string(),
                process_id: process.uuid(),
            });
        }

        // Stat-only health check before launch (uses MinecraftHealthService).
        // Checks every library, client.jar, Java binary, and asset index.
        // On network error (offline), unwrap_or(true) skips the check.
        // If something is missing, the full install runs with a progress bar.
        if instance.install_stage != InstanceInstallStage::Installed
            || !self
                .minecraft_health_service
                .verify_files(MinecraftHealthParams {
                    game_version: instance.game_version.clone(),
                    loader: instance.loader,
                    loader_version: instance.loader_version.clone(),
                    java_path: if instance.java_path.data.is_empty() {
                        None
                    } else {
                        Some(instance.java_path.data.clone())
                    },
                })
                .await
                .unwrap_or(true)
        {
            self.instance_install_service
                .execute(instance_id.clone(), false, None)
                .await?;
        }

        let pre_launch_command = launch_settings.hooks.pre_launch();

        let instance_path = self.location_info.instance_dir(instance.id());

        if let Ok(cmd) = SerializableCommand::from_string(pre_launch_command, Some(&instance_path))
        {
            let result = cmd
                .to_tokio_command()
                .spawn()
                .map_err(|e| {
                    InstanceError::Storage(IoError::with_path(e, &instance_path).to_string())
                })?
                .wait()
                .await
                .map_err(|err| InstanceError::Storage(err.to_string()))?;

            if !result.success() {
                return Err(InstanceError::PrelaunchCommandError {
                    code: result.code().unwrap_or(-1),
                });
            }
        }

        self.notify_plugin_event(
            &instance,
            PluginInternalEvent::BeforeInstanceLaunch {
                instance_id: instance.id().clone(),
            },
            "BeforeInstanceLaunch",
        )
        .await;

        let command = self
            .minecraft_launch_command_service
            .execute(
                GetMinecraftLaunchCommandParams {
                    game_version: instance.game_version.clone(),
                    loader: instance.loader,
                    loader_version: instance.loader_version.clone(),
                    launch_dir: instance_path,
                    java_path: if instance.java_path.data.is_empty() {
                        None
                    } else {
                        Some(instance.java_path.data.clone())
                    },
                },
                launch_settings.clone(),
                credentials,
            )
            .await?;

        self.instance_storage
            .upsert_with(instance.id(), |instance| {
                instance.last_played = Some(chrono::Utc::now());
                Ok(())
            })
            .await?;

        let metadata = self
            .process_start_service
            .execute(
                instance_id,
                command,
                launch_settings.hooks.post_exit().to_owned(),
            )
            .await;

        self.notify_plugin_event(
            &instance,
            PluginInternalEvent::AfterInstanceLaunch {
                instance_id: instance.id().clone(),
            },
            "AfterInstanceLaunch",
        )
        .await;

        Ok(metadata?)
    }
}

#[async_trait]
impl InstanceLaunchService for LaunchInstanceUseCase {
    async fn execute(
        &self,
        instance_id: InstanceId,
        credentials: Credential,
    ) -> Result<MinecraftProcessMetadata, InstanceError> {
        self.execute(instance_id, credentials).await
    }
}
