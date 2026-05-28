use std::{path::Path, sync::Arc};

use tracing::debug;

use crate::features::{
    events::ProgressBarId,
    java::{
        GetJavaUseCase, InstallJava, InstallJavaUseCase, Java, JavaApplicationError,
        JavaInstallationService, JavaInstallationTracker, JavaStorage, JreProvider,
    },
    minecraft::{
        GetVersionManifestUseCase, InstallMinecraftParams, LoaderVersionResolver, MetadataStorage,
        MinecraftApplicationError, MinecraftDomainError, MinecraftDownloader, ModLoader,
        ModLoaderProcessor, get_compatible_java_version, resolve_minecraft_version, vanilla,
    },
};

pub struct InstallMinecraftUseCase<
    MS: MetadataStorage,
    MD: MinecraftDownloader,
    MLP: ModLoaderProcessor,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    JP: JreProvider,
    JIT: JavaInstallationTracker,
> {
    loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
    get_version_manifest_use_case: Arc<GetVersionManifestUseCase<MS>>,
    minecraft_download_service: MD,
    mod_loader_processor: Arc<MLP>,
    java_installation_service: JIS,
    get_java_use_case: Arc<GetJavaUseCase<JS, JIS>>,
    install_java_use_case: Arc<InstallJavaUseCase<JS, JIS, JP, JIT>>,
}

impl<
    MS: MetadataStorage,
    MD: MinecraftDownloader,
    MLP: ModLoaderProcessor,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    JP: JreProvider,
    JIT: JavaInstallationTracker,
> InstallMinecraftUseCase<MS, MD, MLP, JIS, JS, JP, JIT>
{
    // TODO: try to decrease arguments count
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
        get_version_manifest_use_case: Arc<GetVersionManifestUseCase<MS>>,
        minecraft_download_service: MD,
        mod_loader_processor: Arc<MLP>,
        java_installation_service: JIS,
        get_java_use_case: Arc<GetJavaUseCase<JS, JIS>>,
        install_java_use_case: Arc<InstallJavaUseCase<JS, JIS, JP, JIT>>,
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
