use std::{path::Path, sync::Arc, time::Duration};

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
    shared::serializable_command::domain::SerializableCommand,
};

/// How long a pre-launch hook may run. The instance stays locked meanwhile, and a hook that
/// never ends would keep the game from starting for good.
const PRELAUNCH_HOOK_TIMEOUT: Duration = Duration::from_mins(10);

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
        let mut instance = self.instance_storage.get(&instance_id).await?;

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
                    java_path: instance.java_path_override(),
                })
                .await
                .unwrap_or(true)
        {
            self.instance_install_service
                .execute(instance_id.clone(), false, None)
                .await?;
            // The install may have pinned the loader version.
            instance = self.instance_storage.get(&instance_id).await?;
        }

        let pre_launch_command = launch_settings.hooks.pre_launch();

        let instance_path = self.location_info.instance_dir(instance.id());

        run_pre_launch_hook(pre_launch_command, &instance_path, PRELAUNCH_HOOK_TIMEOUT).await?;

        self.notify_plugin_event(
            &instance,
            PluginInternalEvent::BeforeInstanceLaunch {
                instance_id: instance.id().clone(),
            },
            "BeforeInstanceLaunch",
        )
        .await;

        let launch_command = self
            .minecraft_launch_command_service
            .execute(
                GetMinecraftLaunchCommandParams {
                    game_version: instance.game_version.clone(),
                    loader: instance.loader,
                    loader_version: instance.loader_version.clone(),
                    launch_dir: instance_path,
                    java_path: instance.java_path_override(),
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
                launch_command.command,
                launch_settings.hooks.post_exit().to_owned(),
                launch_command.secrets,
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

/// Runs the pre-launch hook `command` in `dir` and waits for it for `timeout`. An empty hook is
/// no hook. A hook still running at the deadline is killed: the game does not start under it.
async fn run_pre_launch_hook(
    command: &str,
    dir: &Path,
    timeout: Duration,
) -> Result<(), InstanceError> {
    if command.trim().is_empty() {
        return Ok(());
    }
    let command = SerializableCommand::from_string(command, Some(dir))
        .map_err(|reason| InstanceError::PrelaunchCommandInvalid { reason })?;
    run_hook_command(&command, timeout).await
}

async fn run_hook_command(
    command: &SerializableCommand,
    timeout: Duration,
) -> Result<(), InstanceError> {
    let mut child = command
        .to_tokio_command()
        .kill_on_drop(true)
        .spawn()
        .map_err(|source| InstanceError::PrelaunchCommandSpawnFailed {
            program: command.program.clone(),
            source,
        })?;

    let Ok(status) = tokio::time::timeout(timeout, child.wait()).await else {
        if let Err(err) = child.kill().await {
            tracing::warn!("Failed to kill the timed out pre-launch hook: {err}");
        }
        return Err(InstanceError::PrelaunchCommandTimeout {
            seconds: timeout.as_secs(),
        });
    };
    let status = status.map_err(InstanceError::storage)?;

    if status.success() {
        Ok(())
    } else {
        Err(InstanceError::PrelaunchCommandError {
            code: status.code().unwrap_or(-1),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Not a test: the process the hook tests start (this very binary), which never ends.
    #[test]
    #[ignore = "started by the pre-launch hook tests as a process that hangs"]
    fn hang_as_a_hook() {
        std::thread::sleep(Duration::from_secs(600));
    }

    fn hanging_hook() -> SerializableCommand {
        let module = module_path!().trim_start_matches("aether_core::");
        SerializableCommand {
            program: std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            args: vec![
                "--ignored".to_owned(),
                "--exact".to_owned(),
                format!("{module}::hang_as_a_hook"),
            ],
            current_dir: None,
            env: None,
        }
    }

    #[tokio::test]
    async fn should_skip_the_hook_when_it_is_empty_or_blank() {
        for command in [
            "", "   ", "	
",
        ] {
            let result = run_pre_launch_hook(command, Path::new("."), Duration::from_secs(1)).await;
            assert!(result.is_ok(), "{command:?}: {result:?}");
        }
    }

    #[tokio::test]
    async fn should_fail_when_the_hook_cannot_be_started() {
        let result = run_pre_launch_hook(
            "aether-no-such-program-0f3a",
            Path::new("."),
            Duration::from_secs(1),
        )
        .await;

        assert!(
            matches!(
                result,
                Err(InstanceError::PrelaunchCommandSpawnFailed { .. })
            ),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn should_time_out_when_the_hook_does_not_end() {
        let started = std::time::Instant::now();

        let result = run_hook_command(&hanging_hook(), Duration::from_millis(500)).await;

        assert!(
            matches!(result, Err(InstanceError::PrelaunchCommandTimeout { .. })),
            "{result:?}"
        );
        assert!(started.elapsed() < Duration::from_secs(30));
    }
}
