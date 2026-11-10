use std::sync::Arc;

use async_trait::async_trait;
use tracing::debug;

use crate::features::{
    events::{ProgressBarId, ProgressService, ProgressServiceExt},
    java::{
        InstallJava, JavaApplicationError, JavaInstallService, JavaInstallationService,
        JavaQueryService,
    },
    minecraft::{
        GameDownloadParams, GameVersion, InstallMinecraftParams, LoaderVersionPreference,
        LoaderVersionService, MinecraftApplicationError, MinecraftDownloader,
        MinecraftInstallService, ModLoader, ModLoaderProcessor, VersionManifestService,
        get_compatible_java_version, resolve_minecraft_version, version_jar_name,
    },
};

/// The part of the bar of an install (out of 100) that the shared files of its version take,
/// whoever downloads them: the rest is the version's metadata (5), the loader's processors
/// (30, when it has any) and finishing (1).
const GAME_FILES_SPAN: f64 = 94.0;
const GAME_FILES_SPAN_WITH_PROCESSORS: f64 = 64.0;

/// A plan may leave the loader version out: it is the default one then, as for a new instance.
async fn preference_or_default(
    service: &dyn LoaderVersionService,
    game_version: &GameVersion,
    loader: &ModLoader,
    preference: Option<LoaderVersionPreference>,
) -> Result<Option<LoaderVersionPreference>, MinecraftApplicationError> {
    if preference.is_some() || *loader == ModLoader::Vanilla {
        return Ok(preference);
    }
    service
        .try_get_default(game_version, loader)
        .await
        .map_err(MinecraftApplicationError::Domain)
}

struct GameFiles {
    version_info: crate::features::minecraft::vanilla::VersionInfo,
    java: crate::features::java::Java,
    version_jar: String,
}

pub struct InstallMinecraftUseCase {
    loader_version_resolver: Arc<dyn LoaderVersionService>,
    get_version_manifest_use_case: Arc<dyn VersionManifestService>,
    minecraft_download_service: Arc<dyn MinecraftDownloader>,
    mod_loader_processor: Arc<dyn ModLoaderProcessor>,
    java_installation_service: Arc<dyn JavaInstallationService>,
    get_java_use_case: Arc<dyn JavaQueryService>,
    install_java_use_case: Arc<dyn JavaInstallService>,
    progress_service: Arc<dyn ProgressService>,
}

impl InstallMinecraftUseCase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        loader_version_resolver: Arc<dyn LoaderVersionService>,
        get_version_manifest_use_case: Arc<dyn VersionManifestService>,
        minecraft_download_service: Arc<dyn MinecraftDownloader>,
        mod_loader_processor: Arc<dyn ModLoaderProcessor>,
        java_installation_service: Arc<dyn JavaInstallationService>,
        get_java_use_case: Arc<dyn JavaQueryService>,
        install_java_use_case: Arc<dyn JavaInstallService>,
        progress_service: Arc<dyn ProgressService>,
    ) -> Self {
        Self {
            loader_version_resolver,
            get_version_manifest_use_case,
            minecraft_download_service,
            mod_loader_processor,
            java_installation_service,
            get_java_use_case,
            install_java_use_case,
            progress_service,
        }
    }

    /// The version's files and the Java it runs on, downloaded and in place. `mirror_bar` shows
    /// the shared downloads waited on; `progress_bar` is the install's own, advanced as the
    /// steps finish (none for a download ahead of an install).
    async fn download_game_files(
        &self,
        game: GameDownloadParams,
        mirror_bar: Option<&ProgressBarId>,
        progress_bar: Option<&ProgressBarId>,
        force: bool,
    ) -> Result<GameFiles, MinecraftApplicationError> {
        let GameDownloadParams {
            game_version,
            loader,
            loader_version,
            java_path,
        } = game;

        let version_manifest = self.get_version_manifest_use_case.execute().await?;

        let (version, minecraft_updated) =
            resolve_minecraft_version(&game_version, &version_manifest)?;

        let loader_version = preference_or_default(
            self.loader_version_resolver.as_ref(),
            &game_version,
            &loader,
            loader_version,
        )
        .await?;
        let loader_version = self
            .loader_version_resolver
            .resolve(&game_version, &loader, loader_version.as_ref())
            .await?;

        let version_jar = version_jar_name(
            &version.id,
            loader_version.as_ref().map(|it| it.id.as_str()),
        );

        debug!("Java path: {java_path:#?}");

        let version_info = self
            .minecraft_download_service
            .get_version_info(&version, loader_version.as_ref(), Some(force), progress_bar)
            .await?;

        let java = if let Some(java_path) = java_path.as_ref() {
            self.java_installation_service
                .locate_java(java_path.as_path())
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
                    .execute(InstallJava::new(compatible_java_version), mirror_bar)
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
                mirror_bar,
            )
            .await?;
        if let Some(progress_bar) = progress_bar {
            // The files are in: the mirror of their download showed how it went.
            let has_processors = version_info
                .processors
                .as_ref()
                .is_some_and(|processors| !processors.is_empty());
            let span = if has_processors {
                GAME_FILES_SPAN_WITH_PROCESSORS
            } else {
                GAME_FILES_SPAN
            };
            self.progress_service
                .emit_progress_safe(progress_bar, span, None)
                .await;
        }

        Ok(GameFiles {
            version_info,
            java,
            version_jar,
        })
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

        let GameFiles {
            mut version_info,
            java,
            version_jar,
        } = self
            .download_game_files(
                GameDownloadParams {
                    game_version: game_version.clone(),
                    loader,
                    loader_version,
                    java_path,
                },
                loading_bar,
                loading_bar,
                force,
            )
            .await?;

        match loader {
            ModLoader::NeoForge | ModLoader::Forge => {
                self.mod_loader_processor
                    .run(
                        game_version,
                        version_jar,
                        &install_dir,
                        &mut version_info,
                        &java,
                        loading_bar,
                    )
                    .await?;
            }
            ModLoader::Vanilla | ModLoader::Fabric | ModLoader::Quilt => {}
        }

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

    async fn download_game(
        &self,
        params: GameDownloadParams,
        waiting_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftApplicationError> {
        self.download_game_files(params, waiting_bar, None, false)
            .await
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::minecraft::{LoaderVersionId, MinecraftDomainError, modded};

    struct Defaults;

    #[async_trait]
    impl LoaderVersionService for Defaults {
        async fn resolve(
            &self,
            _: &GameVersion,
            _: &ModLoader,
            _: Option<&LoaderVersionPreference>,
        ) -> Result<Option<modded::LoaderVersion>, MinecraftDomainError> {
            unimplemented!("only the default is asked for")
        }

        async fn try_get_default(
            &self,
            _: &GameVersion,
            _: &ModLoader,
        ) -> Result<Option<LoaderVersionPreference>, MinecraftDomainError> {
            Ok(Some(LoaderVersionPreference::Stable))
        }
    }

    async fn preference_of(
        loader: ModLoader,
        pinned: Option<LoaderVersionPreference>,
    ) -> Option<LoaderVersionPreference> {
        preference_or_default(
            &Defaults,
            &GameVersion::from_static("1.21.1"),
            &loader,
            pinned,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn should_take_the_default_loader_version_when_a_modded_plan_pins_none() {
        assert_eq!(
            preference_of(ModLoader::Fabric, None).await,
            Some(LoaderVersionPreference::Stable)
        );
    }

    #[tokio::test]
    async fn should_keep_the_pinned_loader_version_when_the_plan_pins_one() {
        let pinned = LoaderVersionPreference::Exact(LoaderVersionId::from_static("0.16.0"));

        assert_eq!(
            preference_of(ModLoader::Fabric, Some(pinned.clone())).await,
            Some(pinned)
        );
    }

    #[tokio::test]
    async fn should_pick_no_loader_version_when_the_game_is_vanilla() {
        assert_eq!(preference_of(ModLoader::Vanilla, None).await, None);
    }
}
