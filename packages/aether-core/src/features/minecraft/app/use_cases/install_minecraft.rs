use std::sync::Arc;

use async_trait::async_trait;
use tracing::debug;

use crate::{
    features::{
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
    },
    shared::keyed_locks::KeyedLockGuard,
};

/// The part of the bar of an install (out of 100) that the shared files of its version take,
/// whoever downloads them: the rest is the version's metadata (5), the loader's processors
/// (30, when it has any) and finishing (1).
const GAME_FILES_SPAN: f64 = 94.0;
const GAME_FILES_SPAN_WITH_PROCESSORS: f64 = 64.0;

/// The default loader version when none is pinned, as for a new instance.
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
    /// Held while the loader's processors run: they write the version's shared files, which no
    /// other install of the same version may download or process meanwhile. Other builds of the
    /// same Minecraft version hold their own lock and can still share some processor outputs.
    version_lock: KeyedLockGuard,
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

        let version_lock = self
            .minecraft_download_service
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
            version_lock,
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
            version_lock,
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
        drop(version_lock);

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
        // Unlike an installed instance, a plan may leave the loader version out.
        let loader_version = preference_or_default(
            self.loader_version_resolver.as_ref(),
            &params.game_version,
            &params.loader,
            params.loader_version,
        )
        .await?;
        let params = GameDownloadParams {
            loader_version,
            ..params
        };
        // No processors run ahead of an install, so the version is let go at once.
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

    // ---- the version's lock over the processors ----

    use std::{
        path::Path,
        sync::atomic::{AtomicBool, AtomicUsize, Ordering},
        time::Duration,
    };

    use tokio::sync::{Notify, watch};

    use crate::{
        features::{
            events::{ProgressServiceImpl, infra::InMemoryProgressBarStorage},
            java::{Java, JavaDomainError, JavaPath},
            minecraft::{ProcessorFailure, vanilla},
        },
        shared::keyed_locks::KeyedLocks,
    };

    const GAME: &str = "1.20.1";

    fn java() -> Java {
        let path = std::env::temp_dir().join("java");
        Java::new(
            17,
            "17".to_owned(),
            "x86_64".to_owned(),
            JavaPath::parse(&path.to_string_lossy()).unwrap(),
        )
    }

    /// The game and its Forge, both there: only the version's lock and processors matter.
    struct Forge;

    #[async_trait]
    impl LoaderVersionService for Forge {
        async fn resolve(
            &self,
            _: &GameVersion,
            _: &ModLoader,
            _: Option<&LoaderVersionPreference>,
        ) -> Result<Option<modded::LoaderVersion>, MinecraftDomainError> {
            Ok(Some(modded::LoaderVersion {
                id: "forge-47.0".to_owned(),
                url: String::new(),
                stable: true,
            }))
        }

        async fn try_get_default(
            &self,
            _: &GameVersion,
            _: &ModLoader,
        ) -> Result<Option<LoaderVersionPreference>, MinecraftDomainError> {
            unimplemented!("the install pins its loader version")
        }
    }

    #[async_trait]
    impl VersionManifestService for Forge {
        async fn execute(&self) -> Result<vanilla::VersionManifest, MinecraftApplicationError> {
            Ok(vanilla::VersionManifest {
                latest: vanilla::LatestVersion {
                    release: GAME.to_owned(),
                    snapshot: GAME.to_owned(),
                },
                versions: vec![vanilla::Version {
                    id: GAME.to_owned(),
                    type_: vanilla::VersionType::Release,
                    url: String::new(),
                    time: chrono::DateTime::UNIX_EPOCH,
                    release_time: chrono::DateTime::UNIX_EPOCH,
                    sha1: String::new(),
                    compliance_level: 0,
                    original_sha1: None,
                }],
            })
        }
    }

    #[async_trait]
    impl JavaInstallationService for Forge {
        async fn locate_java(&self, _: &Path) -> Result<Java, JavaDomainError> {
            Ok(java())
        }

        async fn discover_installations(
            &self,
            _: &[std::path::PathBuf],
        ) -> Result<Vec<Java>, JavaDomainError> {
            unimplemented!("the install names its Java")
        }
    }

    #[async_trait]
    impl JavaQueryService for Forge {
        async fn execute(&self, _: u32) -> Result<Java, JavaApplicationError> {
            unimplemented!("the install names its Java")
        }
    }

    #[async_trait]
    impl JavaInstallService for Forge {
        async fn execute(
            &self,
            _: InstallJava,
            _: Option<&ProgressBarId>,
        ) -> Result<Java, JavaApplicationError> {
            unimplemented!("the install names its Java")
        }
    }

    /// Has every file of a version already, and locks the version as the real downloader does.
    #[derive(Default)]
    struct Downloader {
        locks: KeyedLocks<String>,
        /// How many installs have asked for the version's files.
        asked: AtomicUsize,
    }

    #[async_trait]
    impl MinecraftDownloader for Downloader {
        async fn download_minecraft(
            &self,
            version_info: &vanilla::VersionInfo,
            _: &str,
            _: bool,
            _: bool,
            _: Option<&ProgressBarId>,
        ) -> Result<KeyedLockGuard, MinecraftDomainError> {
            self.asked.fetch_add(1, Ordering::SeqCst);
            Ok(self.locks.lock(&version_info.id).await)
        }

        async fn get_version_info(
            &self,
            version: &vanilla::Version,
            loader: Option<&modded::LoaderVersion>,
            _: Option<bool>,
            _: Option<&ProgressBarId>,
        ) -> Result<vanilla::VersionInfo, MinecraftDomainError> {
            let id = version_jar_name(&version.id, loader.map(|loader| loader.id.as_str()));
            Ok(serde_json::from_value(serde_json::json!({
                "assetIndex": { "id": "1", "sha1": "0", "size": 1, "totalSize": 1, "url": "" },
                "assets": "1",
                "downloads": {},
                "id": id,
                "javaVersion": null,
                "libraries": [],
                "mainClass": "Main",
                "minimumLauncherVersion": 1,
                "releaseTime": "2020-01-01T00:00:00Z",
                "time": "2020-01-01T00:00:00Z",
                "type": "release",
                "data": {},
                "processors": [{ "jar": "a:b:1", "classpath": [], "args": [] }],
            }))
            .unwrap())
        }
    }

    /// Writes the version's outputs once they are not there, as the real processors do: until
    /// the gate opens, then it skips them. Counts the runs, and how many ran at once.
    struct Processors {
        done: AtomicBool,
        runs: AtomicUsize,
        running: AtomicUsize,
        most_at_once: AtomicUsize,
        started: Notify,
        gate: watch::Sender<bool>,
        /// The next run fails once through the gate.
        fails: AtomicBool,
    }

    impl Default for Processors {
        fn default() -> Self {
            Self {
                done: AtomicBool::default(),
                runs: AtomicUsize::default(),
                running: AtomicUsize::default(),
                most_at_once: AtomicUsize::default(),
                started: Notify::new(),
                gate: watch::channel(false).0,
                fails: AtomicBool::default(),
            }
        }
    }

    #[async_trait]
    impl ModLoaderProcessor for Processors {
        async fn run(
            &self,
            _: GameVersion,
            _: String,
            _: &Path,
            _: &mut vanilla::VersionInfo,
            _: &Java,
            _: Option<&ProgressBarId>,
        ) -> Result<(), MinecraftDomainError> {
            if self.done.load(Ordering::SeqCst) {
                return Ok(());
            }
            self.runs.fetch_add(1, Ordering::SeqCst);
            let now = self.running.fetch_add(1, Ordering::SeqCst) + 1;
            self.most_at_once.fetch_max(now, Ordering::SeqCst);
            self.started.notify_one();
            self.gate.subscribe().wait_for(|open| *open).await.unwrap();
            self.running.fetch_sub(1, Ordering::SeqCst);
            if self.fails.swap(false, Ordering::SeqCst) {
                return Err(MinecraftDomainError::ProcessorFailed {
                    reason: ProcessorFailure::Exited {
                        code: Some(1),
                        stderr: String::new(),
                    },
                });
            }
            self.done.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    struct Installs {
        use_case: Arc<InstallMinecraftUseCase>,
        downloader: Arc<Downloader>,
        processors: Arc<Processors>,
    }

    fn installs() -> Installs {
        let downloader = Arc::new(Downloader::default());
        let processors = Arc::new(Processors::default());
        let progress = Arc::new(ProgressServiceImpl::new(
            Arc::new(crate::features::events::test_support::NoOpEmitter),
            Arc::new(InMemoryProgressBarStorage::default()),
        ));
        let use_case = Arc::new(InstallMinecraftUseCase::new(
            Arc::new(Forge),
            Arc::new(Forge),
            downloader.clone(),
            processors.clone(),
            Arc::new(Forge),
            Arc::new(Forge),
            Arc::new(Forge),
            progress,
        ));
        Installs {
            use_case,
            downloader,
            processors,
        }
    }

    impl Installs {
        fn install(&self) -> tokio::task::JoinHandle<Result<(), MinecraftApplicationError>> {
            let use_case = self.use_case.clone();
            let params = InstallMinecraftParams {
                game_version: GameVersion::from_static(GAME),
                loader: ModLoader::Forge,
                loader_version: Some(LoaderVersionPreference::Exact(
                    LoaderVersionId::from_static("forge-47.0"),
                )),
                install_dir: std::env::temp_dir(),
                java_path: Some(java().path().clone()),
            };
            tokio::spawn(async move { use_case.execute(params, None, false).await })
        }

        /// Until `count` installs have asked for the version, and a while more: long enough for
        /// one that is not held back to reach the processors.
        async fn asked(&self, count: usize) {
            tokio::time::timeout(Duration::from_secs(5), async {
                while self.downloader.asked.load(Ordering::SeqCst) < count {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .expect("the installs never asked for the version");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    #[tokio::test]
    async fn should_run_the_processors_once_when_two_installs_of_a_version_run_together() {
        let installs = installs();
        let first = installs.install();
        installs.processors.started.notified().await;

        let second = installs.install();
        installs.asked(2).await;
        installs.processors.gate.send_replace(true);

        first.await.unwrap().unwrap();
        second.await.unwrap().unwrap();
        assert_eq!(installs.processors.runs.load(Ordering::SeqCst), 1);
        assert_eq!(installs.processors.most_at_once.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn should_let_the_version_go_when_its_processors_fail() {
        let installs = installs();
        installs.processors.fails.store(true, Ordering::SeqCst);
        let first = installs.install();
        installs.processors.started.notified().await;
        let second = installs.install();
        installs.asked(2).await;

        installs.processors.gate.send_replace(true);

        assert!(first.await.unwrap().is_err());
        tokio::time::timeout(Duration::from_secs(5), second)
            .await
            .expect("the version was kept after its processors failed")
            .unwrap()
            .unwrap();
        assert_eq!(installs.processors.runs.load(Ordering::SeqCst), 2);
        assert_eq!(installs.processors.most_at_once.load(Ordering::SeqCst), 1);
    }
}
