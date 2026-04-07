use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::{
    features::{
        auth::Credentials,
        java::{
            app::{GetJavaUseCase, JavaApplicationError},
            JavaInstallationService, JavaStorage,
        },
        minecraft::{
            app::{GetVersionManifestUseCase, MinecraftApplicationError},
            resolve_minecraft_version,
            utils::get_compatible_java_version,
            LaunchSettings, LoaderVersionPreference, LoaderVersionResolver, MetadataStorage,
            MinecraftDownloader, ModLoader,
        },
        settings::LocationInfo,
    },
    shared::create_dir_all,
    with_mut_ref,
};

use super::{
    get_minecraft_arguments::get_minecraft_arguments,
    get_minecraft_jvm_arguments::get_minecraft_jvm_arguments,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMinecraftLaunchCommandParams {
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,
    pub launch_dir: PathBuf,
    pub java_path: Option<String>,
}

pub struct GetMinecraftLaunchCommandUseCase<
    MS: MetadataStorage,
    MD: MinecraftDownloader,
    JIS: JavaInstallationService,
    JS: JavaStorage,
> {
    loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
    get_version_manifest_use_case: Arc<GetVersionManifestUseCase<MS>>,
    minecraft_downloader: MD,
    java_installation_service: JIS,
    get_java_use_case: Arc<GetJavaUseCase<JS, JIS>>,
    location_info: Arc<LocationInfo>,
}

impl<
        MS: MetadataStorage,
        MD: MinecraftDownloader,
        JIS: JavaInstallationService,
        JS: JavaStorage,
    > GetMinecraftLaunchCommandUseCase<MS, MD, JIS, JS>
{
    pub fn new(
        loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
        get_version_manifest_use_case: Arc<GetVersionManifestUseCase<MS>>,
        minecraft_downloader: MD,
        java_installation_service: JIS,
        get_java_use_case: Arc<GetJavaUseCase<JS, JIS>>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            loader_version_resolver,
            get_version_manifest_use_case,
            minecraft_downloader,
            java_installation_service,
            get_java_use_case,
            location_info,
        }
    }

    pub async fn execute(
        &self,
        get_minecraft_launch_command_params: GetMinecraftLaunchCommandParams,
        launch_settings: LaunchSettings,
        credentials: Credentials,
    ) -> Result<Command, MinecraftApplicationError> {
        let GetMinecraftLaunchCommandParams {
            game_version,
            loader,
            loader_version,
            launch_dir,
            java_path,
        } = get_minecraft_launch_command_params;

        let version_manifest = self.get_version_manifest_use_case.execute().await?;

        let (version, minecraft_updated) =
            resolve_minecraft_version(&game_version, version_manifest)?;

        let loader_version = self
            .loader_version_resolver
            .resolve(&game_version, &loader, loader_version.as_ref())
            .await?;

        let version_jar = loader_version.as_ref().map_or(version.id.clone(), |it| {
            format!("{}-{}", version.id.clone(), it.id.clone())
        });

        let version_info = self
            .minecraft_downloader
            .get_version_info(&version, loader_version.as_ref(), None, None)
            .await?;

        let java = if let Some(java_path) = java_path.as_ref() {
            self.java_installation_service
                .locate_java(Path::new(java_path))
                .await
                .map_err(|err| {
                    MinecraftApplicationError::JavaError(JavaApplicationError::Domain(err))
                })
        } else {
            let compatible_java_version = get_compatible_java_version(&version_info);
            self.get_java_use_case
                .execute(compatible_java_version)
                .await
                .map_err(Into::into)
        }?;

        // TODO: refactor
        let client_path = self
            .location_info
            .version_dir(&version_jar)
            .join(format!("{version_jar}.jar"));

        let args = version_info.arguments.clone().unwrap_or_default();

        let env_args_vec = launch_settings.env_vars.clone();

        let natives_dir = self.location_info.version_natives_dir(&version_jar);
        if !natives_dir.exists() {
            create_dir_all(&natives_dir).await?;
        }

        let jvm_arguments = get_minecraft_jvm_arguments(
            args.get(&daedalus::minecraft::ArgumentType::Jvm)
                .map(|x| x.as_slice()),
            &self.location_info.libraries_dir(),
            &version_info,
            &natives_dir,
            &client_path,
            version_jar,
            &java,
            launch_settings.memory.maximum,
            &launch_settings.launch_args,
            minecraft_updated,
        )?;

        let minecraft_arguments = get_minecraft_arguments(
            args.get(&daedalus::minecraft::ArgumentType::Game)
                .map(|x| x.as_slice()),
            version_info.minecraft_arguments.as_deref(),
            &credentials,
            &version.id,
            &version_info.asset_index.id,
            &launch_dir,
            &self.location_info.assets_dir(),
            &version.type_,
            launch_settings.game_resolution,
            java.architecture(),
        )?
        .into_iter()
        .collect::<Vec<_>>();

        let mut command = match &launch_settings.hooks.wrapper() {
            Some(hook) => {
                with_mut_ref!(it = tokio::process::Command::new(hook) => {it.arg(java.path())})
            }
            None => tokio::process::Command::new(java.path()),
        };

        command
            .args(jvm_arguments)
            .arg(version_info.main_class.clone())
            .args(minecraft_arguments)
            .current_dir(launch_dir.clone());

        // CARGO-set DYLD_LIBRARY_PATH breaks Minecraft on macOS during testing on playground
        #[cfg(target_os = "macos")]
        if std::env::var("CARGO").is_ok() {
            command.env_remove("DYLD_FALLBACK_LIBRARY_PATH");
        }

        // Java options should be set in instance options (the existence of _JAVA_OPTIONS overwrites them)
        command.env_remove("_JAVA_OPTIONS");

        command.envs(env_args_vec);

        // options.txt override

        Ok(command)
    }
}
