use std::sync::Arc;

use crate::{
    features::{
        auth::Credentials,
        events::{EventEmitter, ProgressService},
        instance::{
            Instance, InstanceError, InstanceInstallStage, InstanceStorage, InstanceStorageExt,
        },
        java::{JavaInstallationService, JavaStorage, JreProvider},
        minecraft::{
            app::{GetMinecraftLaunchCommandParams, GetMinecraftLaunchCommandUseCase},
            LaunchSettings, MetadataStorage, MinecraftDownloader,
        },
        process::{
            app::{GetProcessMetadataByInstanceIdUseCase, StartProcessUseCase},
            MinecraftProcessMetadata, ProcessStorage,
        },
        settings::{DefaultInstanceSettings, DefaultInstanceSettingsStorage, Hooks, LocationInfo},
    },
    shared::{IoError, SerializableCommand},
};

use super::InstallInstanceUseCase;

pub struct LaunchInstanceUseCase<
    IS: InstanceStorage,
    MS: MetadataStorage,
    PS: ProcessStorage,
    GISS: DefaultInstanceSettingsStorage,
    E: EventEmitter,
    MD: MinecraftDownloader,
    PGS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    JP: JreProvider,
> {
    instance_storage: Arc<IS>,
    default_instance_settings_storage: Arc<GISS>,
    location_info: Arc<LocationInfo>,
    get_process_by_instance_id_use_case: Arc<GetProcessMetadataByInstanceIdUseCase<PS>>,
    install_instance_use_case: Arc<InstallInstanceUseCase<IS, MS, MD, PGS, JIS, JS, JP>>,
    get_minecraft_launch_command_use_case: GetMinecraftLaunchCommandUseCase<MS, MD, JIS, JS>,
    start_process_use_case: Arc<StartProcessUseCase<E, PS, IS>>,
}

impl<
        IS: InstanceStorage + 'static,
        MS: MetadataStorage,
        PS: ProcessStorage + 'static,
        GISS: DefaultInstanceSettingsStorage,
        E: EventEmitter + 'static,
        MD: MinecraftDownloader,
        PGS: ProgressService,
        JIS: JavaInstallationService,
        JS: JavaStorage,
        JP: JreProvider,
    > LaunchInstanceUseCase<IS, MS, PS, GISS, E, MD, PGS, JIS, JS, JP>
{
    pub fn new(
        instance_storage: Arc<IS>,
        default_instance_settings_storage: Arc<GISS>,
        location_info: Arc<LocationInfo>,
        get_process_by_instance_id_use_case: Arc<GetProcessMetadataByInstanceIdUseCase<PS>>,
        install_instance_use_case: Arc<InstallInstanceUseCase<IS, MS, MD, PGS, JIS, JS, JP>>,
        get_minecraft_launch_command_use_case: GetMinecraftLaunchCommandUseCase<MS, MD, JIS, JS>,
        start_process_use_case: Arc<StartProcessUseCase<E, PS, IS>>,
    ) -> Self {
        Self {
            instance_storage,
            default_instance_settings_storage,
            location_info,
            get_process_by_instance_id_use_case,
            install_instance_use_case,
            get_minecraft_launch_command_use_case,
            start_process_use_case,
        }
    }

    fn resolve_launch_settings(
        instance: &Instance,
        settings: &DefaultInstanceSettings,
    ) -> LaunchSettings {
        LaunchSettings {
            launch_args: instance
                .launch_args
                .clone()
                .unwrap_or_else(|| settings.launch_args().to_vec()),

            env_vars: instance
                .env_vars
                .clone()
                .unwrap_or_else(|| settings.env_vars().to_vec()),

            memory: instance.memory.unwrap_or(settings.memory()),

            game_resolution: instance
                .game_resolution
                .unwrap_or(settings.game_resolution()),

            hooks: Hooks::new(
                instance
                    .hooks
                    .pre_launch()
                    .cloned()
                    .or_else(|| settings.hooks().pre_launch().cloned()),
                instance
                    .hooks
                    .wrapper()
                    .cloned()
                    .or_else(|| settings.hooks().wrapper().cloned()),
                instance
                    .hooks
                    .post_exit()
                    .cloned()
                    .or_else(|| settings.hooks().post_exit().cloned()),
            ),
        }
    }

    pub async fn execute(
        &self,
        instance_id: String,
        credentials: Credentials,
    ) -> Result<MinecraftProcessMetadata, InstanceError> {
        let settings = self.default_instance_settings_storage.get().await?;
        let instance = self.instance_storage.get(&instance_id).await?;

        let launch_settings = Self::resolve_launch_settings(&instance, &settings);

        let instance = self.instance_storage.get(&instance_id).await?;

        if instance.install_stage == InstanceInstallStage::PackInstalling
            || instance.install_stage == InstanceInstallStage::Installing
        {
            return Err(InstanceError::InstanceStillInstalling { instance_id });
        }

        // Check if profile has a running profile, and reject running the command if it does
        // Done late so a quick double call doesn't launch two instances
        if let Some(process) = self
            .get_process_by_instance_id_use_case
            .execute(instance.id.clone())
            .await?
            .first()
        {
            return Err(InstanceError::InstanceAlreadyRunning {
                instance_id,
                process_id: process.uuid(),
            });
        }

        if instance.install_stage != InstanceInstallStage::Installed {
            self.install_instance_use_case
                .execute(instance_id, false)
                .await?;
        }

        let pre_launch_command = instance
            .hooks
            .pre_launch()
            .or(launch_settings.hooks.pre_launch());

        let instance_path = self.location_info.instance_dir(&instance.id);

        if let Some(command) = pre_launch_command {
            if let Ok(cmd) = SerializableCommand::from_string(command, Some(&instance_path)) {
                let result = cmd
                    .to_tokio_command()
                    .spawn()
                    .map_err(|e| IoError::with_path(e, &instance_path))?
                    .wait()
                    .await
                    .map_err(IoError::from)?;

                if !result.success() {
                    return Err(InstanceError::PrelaunchCommandError {
                        code: result.code().unwrap_or(-1),
                    });
                }
            }
        }

        // run_pre_launch_command(&pre_launch_command, &instance_path).await?;

        // let lazy_locator = LazyLocator::get().await?;
        // let plugin_registry = lazy_locator.get_plugin_registry().await;

        // if let Some(pack_info) = &instance.pack_info {
        //     if let Ok(plugin) = plugin_registry.get(&pack_info.pack_type) {
        //         if let Some(plugin) = &plugin.instance {
        //             let mut plugin = plugin.lock().await;
        //             if plugin.supports_handle_events() {
        //                 plugin.handle_event(&PluginEvent::BeforeInstanceLaunch {
        //                     instance_id: instance.id.clone(),
        //                 })?;
        //             }
        //         }
        //     }
        // }

        let command = self
            .get_minecraft_launch_command_use_case
            .execute(
                GetMinecraftLaunchCommandParams {
                    game_version: instance.game_version.clone(),
                    loader: instance.loader,
                    loader_version: instance.loader_version.clone(),
                    launch_dir: instance_path,
                    java_path: instance.java_path.clone(),
                },
                launch_settings.clone(),
                credentials,
            )
            .await?;

        self.instance_storage
            .upsert_with(&instance.id, |instance| {
                instance.last_played = Some(chrono::Utc::now());
                Ok(())
            })
            .await?;

        let metadata = self
            .start_process_use_case
            .execute(
                instance.id.clone(),
                command,
                launch_settings.hooks.post_exit().cloned(),
            )
            .await;

        // if let Some(pack_info) = &instance.pack_info {
        //     if let Ok(plugin) = plugin_registry.get(&pack_info.pack_type) {
        //         if let Some(plugin) = &plugin.instance {
        //             let mut plugin = plugin.lock().await;
        //             if plugin.supports_handle_events() {
        //                 plugin.handle_event(&PluginEvent::AfterInstanceLaunch {
        //                     instance_id: instance.id.clone(),
        //                 })?;
        //             }
        //         }
        //     }
        // }

        Ok(metadata?)
    }
}
