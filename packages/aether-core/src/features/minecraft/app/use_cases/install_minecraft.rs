use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use tracing::debug;

use crate::features::{
    events::ProgressBarId,
    java::{
        InstallJava, Java, JavaApplicationError, JavaInstallService, JavaInstallationService,
        JavaQueryService,
    },
    minecraft::{
        InstallMinecraftParams, LoaderVersionService, MinecraftApplicationError,
        MinecraftDomainError, MinecraftDownloader, MinecraftInstallService, ModLoader,
        ModLoaderProcessor, VersionManifestService, get_compatible_java_version,
        resolve_minecraft_version, vanilla,
    },
};

pub struct InstallMinecraftUseCase {
    loader_version_resolver: Arc<dyn LoaderVersionService>,
    get_version_manifest_use_case: Arc<dyn VersionManifestService>,
    minecraft_download_service: Arc<dyn MinecraftDownloader>,
    mod_loader_processor: Arc<dyn ModLoaderProcessor>,
    java_installation_service: Arc<dyn JavaInstallationService>,
    get_java_use_case: Arc<dyn JavaQueryService>,
    install_java_use_case: Arc<dyn JavaInstallService>,
}

impl InstallMinecraftUseCase {
    // TODO: try to decrease arguments count
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        loader_version_resolver: Arc<dyn LoaderVersionService>,
        get_version_manifest_use_case: Arc<dyn VersionManifestService>,
        minecraft_download_service: Arc<dyn MinecraftDownloader>,
        mod_loader_processor: Arc<dyn ModLoaderProcessor>,
        java_installation_service: Arc<dyn JavaInstallationService>,
        get_java_use_case: Arc<dyn JavaQueryService>,
        install_java_use_case: Arc<dyn JavaInstallService>,
    ) -> Self {
        Self {
            loader_version_resolver,
            get_version_manifest_use_case,
            minecraft_download_service,
            mod_loader_processor,
            java_installation_service,
            get_java_use_case,
            install_java_use_case,
        }
    }

    // TODO: try to decrease arguments count
    #[allow(clippy::too_many_arguments)]
    async fn run_mod_loader_post_install(
        &self,
        game_version: String,
        loader: ModLoader,
        version_jar: String,
        minecraft_dir: &Path,
        version_info: &mut vanilla::VersionInfo,
        java_version: &Java,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError> {
        match loader {
            ModLoader::NeoForge | ModLoader::Forge => {
                self.mod_loader_processor
                    .run(
                        game_version,
                        version_jar,
                        minecraft_dir,
                        version_info,
                        java_version,
                        loading_bar,
                    )
                    .await
            }
            ModLoader::Vanilla | ModLoader::Fabric | ModLoader::Quilt => Ok(()),
        }
    }

    pub async fn execute(
        &self,
        install_minecraft_params: InstallMinecraftParams,
        loading_bar: Option<&ProgressBarId>,
        force: bool,
    ) -> Result<(), MinecraftApplicationError> {
        let InstallMinecraftParams {
            game_version,
            loader,
            loader_version,
            install_dir,
            java_path,
        } = install_minecraft_params;

        let version_manifest = self.get_version_manifest_use_case.execute().await?;

        let (version, minecraft_updated) =
            resolve_minecraft_version(&game_version, &version_manifest)?;

        let loader_version = self
            .loader_version_resolver
            .resolve(&game_version, &loader, loader_version.as_ref())
            .await?;

        let version_jar = loader_version.as_ref().map_or(
            version.id.clone(), // For Vanilla take pure version
            |it| format!("{}-{}", version.id.clone(), it.id.clone()),
        );

        debug!("Java path: {java_path:#?}");

        let mut version_info = self
            .minecraft_download_service
            .get_version_info(&version, loader_version.as_ref(), Some(force), loading_bar)
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

            let java = self
                .get_java_use_case
                .execute(compatible_java_version)
                .await;

            match java {
                Ok(java) => Ok(java),
                Err(_) => self
                    .install_java_use_case
                    .execute(InstallJava::new(compatible_java_version))
                    .await
                    .map_err(Into::into),
            }
        }?;

        self.minecraft_download_service
            .download_minecraft(
                &version_info,
                java.architecture(),
                force,
                minecraft_updated,
                loading_bar,
            )
            .await?;

        self.run_mod_loader_post_install(
            game_version,
            loader,
            version_jar,
            &install_dir,
            &mut version_info,
            &java,
            loading_bar,
        )
        .await?;

        Ok(())
    }
}

#[async_trait]
impl MinecraftInstallService for InstallMinecraftUseCase {
    async fn execute(
        &self,
        params: InstallMinecraftParams,
        loading_bar: Option<&ProgressBarId>,
        force: bool,
    ) -> Result<(), MinecraftApplicationError> {
        self.execute(params, loading_bar, force).await
    }
}
