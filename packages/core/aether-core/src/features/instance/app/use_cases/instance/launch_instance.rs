use std::sync::Arc;

use crate::{
    features::{
        auth::domain::Credential,
        events::ProgressService,
        instance::{
            Instance, InstanceError, InstanceInstallStage, InstanceStorage, InstanceStorageExt,
        },
        java::{JavaInstallationService, JavaInstallationTracker, JavaStorage, JreProvider},
        minecraft::{
            LaunchSettings, MetadataStorage, MinecraftDownloader,
            app::{GetMinecraftLaunchCommandParams, GetMinecraftLaunchCommandUseCase},
        },
        process::{
            MinecraftProcessMetadata, ProcessStorage,
            app::{GetProcessMetadataByInstanceIdUseCase, StartProcessUseCase},
        },
        settings::{DefaultInstanceSettings, DefaultInstanceSettingsStorage, LocationInfo},
    },
    shared::{IoError, SerializableCommand},
};

use super::InstallInstanceUseCase;

pub struct LaunchInstanceUseCase<
    IS: InstanceStorage,
    MS: MetadataStorage,
    PS: ProcessStorage,
    GISS: DefaultInstanceSettingsStorage,
    MD: MinecraftDownloader,
    PGS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    JP: JreProvider,
    JIT: JavaInstallationTracker,
> {
    instance_storage: Arc<IS>,
    default_instance_settings_storage: Arc<GISS>,
    location_info: Arc<LocationInfo>,
    get_process_by_instance_id_use_case: Arc<GetProcessMetadataByInstanceIdUseCase<PS>>,
    #[allow(clippy::type_complexity)]
    install_instance_use_case: Arc<InstallInstanceUseCase<IS, MS, MD, PGS, JIS, JS, JP, JIT>>,
    get_minecraft_launch_command_use_case: GetMinecraftLaunchCommandUseCase<MS, MD, JIS, JS>,
    start_process_use_case: Arc<StartProcessUseCase<PS, IS>>,
}

impl<
    IS: InstanceStorage + 'static,
    MS: MetadataStorage,
    PS: ProcessStorage + 'static,
    GISS: DefaultInstanceSettingsStorage,
    MD: MinecraftDownloader,
    PGS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    JP: JreProvider,
    JIT: JavaInstallationTracker,
> LaunchInstanceUseCase<IS, MS, PS, GISS, MD, PGS, JIS, JS, JP, JIT>
{
    #[allow(clippy::type_complexity)]
    pub fn new(
        instance_storage: Arc<IS>,
        default_instance_settings_storage: Arc<GISS>,
        location_info: Arc<LocationInfo>,
        get_process_by_instance_id_use_case: Arc<GetProcessMetadataByInstanceIdUseCase<PS>>,
        install_instance_use_case: Arc<InstallInstanceUseCase<IS, MS, MD, PGS, JIS, JS, JP, JIT>>,
        get_minecraft_launch_command_use_case: GetMinecraftLaunchCommandUseCase<MS, MD, JIS, JS>,
        start_process_use_case: Arc<StartProcessUseCase<PS, IS>>,
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
            launch_args: instance.launch_args.resolve(settings.launch_args()),
            env_vars: instance.env_vars.resolve(settings.env_vars()),
            memory: instance.memory.resolve(settings.memory()),
            window: instance.window.resolve(settings.window()),
            hooks: instance.hooks.resolve(settings.hooks()),
        }
    }

    pub async fn execute(
        &self,
        instance_id: String,
        credentials: Credential,
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
            .execute(instance_id.clone())
            .await?
            .first()
        {
            return Err(InstanceError::InstanceAlreadyRunning {
                instance_id: instance_id.clone(),
                process_id: process.uuid(),
            });
        }

        if instance.install_stage != InstanceInstallStage::Installed {
            self.install_instance_use_case
                .execute(instance_id.clone(), false)
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

        // run_pre_launch_command(&pre_launch_command, &instance_path).await?;

        // let lazy_locator = LazyLocator::get().await?;
        // let plugin_registry = lazy_locator.get_plugin_registry().await;

        // if let Some(pack_info) = &instance.pack_info {
        //     if let Ok(plugin) = plugin_registry.get(&pack_info.pack_type) {
        //         if let Some(plugin) = &plugin.instance {
        //             let mut plugin = plugin.lock().await;
        //             if plugin.supports_handle_events() {
        //                 plugin.handle_event(&PluginEvent::BeforeInstanceLaunch {
        //                     instance_id: instance.id().clone(),
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
            .start_process_use_case
            .execute(
                instance_id,
                command,
                launch_settings.hooks.post_exit().to_owned(),
            )
            .await;

        // if let Some(pack_info) = &instance.pack_info {
        //     if let Ok(plugin) = plugin_registry.get(&pack_info.pack_type) {
        //         if let Some(plugin) = &plugin.instance {
        //             let mut plugin = plugin.lock().await;
        //             if plugin.supports_handle_events() {
        //                 plugin.handle_event(&PluginEvent::AfterInstanceLaunch {
        //                     instance_id: instance.id().clone(),
        //                 })?;
        //             }
        //         }
        //     }
        // }

        Ok(metadata?)
    }
}
