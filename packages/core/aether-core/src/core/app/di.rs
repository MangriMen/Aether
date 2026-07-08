use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::policies::ExponentialBackoff;

use crate::features::{
    auth::{
        self, ActiveAccountHelper, AuthFeature, CreateOfflineAccountUseCase,
        CreateOfflineAccountUseCasePort, CredentialsStorage, GetAccountsUseCase,
        GetAccountsUseCasePort, LogoutUseCase, LogoutUseCasePort, SetActiveAccountUseCase,
        SetActiveAccountUseCasePort,
        infra::{FsCredentialsStorage, SqliteCredentialsStorage},
    },
    events::{
        Event, EventEmitter, EventEmitterExt, EventsFeature, ListProgressBarsUseCase,
        ListProgressBarsUseCasePort, PluginEvent, ProgressService, ProgressServiceImpl,
        SharedEventEmitter, infra::InMemoryProgressBarStorage,
    },
    file_watcher::{FileEventHandler, FileWatcher, FileWatcherFeature, infra::NotifyFileWatcher},
    instance::{
        self, ChangeContentStateUseCase, ChangeContentStateUseCasePort,
        CheckContentCompatibilityUseCase, CheckContentCompatibilityUseCasePort, ContentFileService,
        ContentProvider, CreateInstanceUseCase, CreateInstanceUseCasePort, EditInstanceIconUseCase,
        EditInstanceIconUseCasePort, EditInstanceUseCase, EditInstanceUseCasePort,
        GetContentUseCase, GetContentUseCasePort, GetInstanceUseCase, GetInstanceUseCasePort,
        ImportContentUseCase, ImportContentUseCasePort, ImportInstanceUseCase,
        ImportInstanceUseCasePort, Importer, InstallContentUseCase, InstallContentUseCasePort,
        InstallInstanceUseCase, InstanceFeature, InstanceFileService, InstanceInstallService,
        InstanceLaunchService, InstanceStorage, InstanceWatcherService, LaunchInstanceUseCase,
        LaunchInstanceWithActiveAccountUseCase, LaunchInstanceWithActiveAccountUseCasePort,
        ListContentUseCase, ListContentUseCasePort, ListContentVersionsUseCase,
        ListContentVersionsUseCasePort, ListImportersUseCase, ListImportersUseCasePort,
        ListInstancesUseCase, ListInstancesUseCasePort, ListProvidersUseCase,
        ListProvidersUseCasePort, PackStorage, RemoveContentUseCase, RemoveContentUseCasePort,
        RemoveInstanceUseCase, RemoveInstanceUseCasePort, SearchContentUseCase,
        SearchContentUseCasePort, UpdateInstanceUseCase, UpdateInstanceUseCasePort, Updater,
        infra::{
            EventEmittingInstanceStorage, FsContentFileService, FsInstanceFileService,
            InstanceEventHandler, InstanceWatcherServiceImpl, ModrinthContentProvider,
            SqliteInstanceStorage, SqlitePackStorage,
        },
    },
    java::{
        self, DiscoverJavaUseCase, DiscoverJavaUseCasePort, EditJavaUseCase, EditJavaUseCasePort,
        GetActiveJavaInstallationsUseCase, GetActiveJavaInstallationsUseCasePort, GetJavaUseCase,
        InstallJavaUseCase, JavaFeature, JavaInstallService, JavaInstallationService,
        JavaInstallationTracker, JavaQueryService, JavaStorage, JreProvider, ListJavaUseCase,
        ListJavaUseCasePort, RemoveJavaUseCase, RemoveJavaUseCasePort, TestJreUseCase,
        TestJreUseCasePort,
        infra::{
            AzulJreProvider, FsJavaInstallationService, MemoryJavaInstallationTracker,
            SqliteJavaStorage, get_default_discovery_paths,
        },
    },
    minecraft::{
        self, GetLoaderVersionManifestUseCase, GetLoaderVersionManifestUseCasePort,
        GetMinecraftLaunchCommandUseCase, GetVersionManifestUseCase, InstallMinecraftUseCase,
        LoaderVersionResolver, LoaderVersionService, MetadataStorage, MinecraftDownloader,
        MinecraftFeature, MinecraftFileHealthService, MinecraftHealthService,
        MinecraftInstallService, MinecraftLaunchCommandService, ModLoaderProcessor,
        VersionManifestService,
        infra::{
            AssetsService, CachedMetadataStorage, ClientService, ForgeProcessor, LibrariesService,
            MinecraftDownloadResolver, MinecraftDownloadService, ModrinthMetadataStorage,
        },
    },
    plugins::{
        CheckForPluginUpdatesUseCase, CheckForPluginUpdatesUseCasePort, DisablePluginUseCase,
        EditPluginSettingsUseCase, EditPluginSettingsUseCasePort, EnablePluginUseCase,
        EnablePluginUseCasePort, ForceEnablePluginUseCase, ForceEnablePluginUseCasePort,
        GetPluginApiVersionUseCase, GetPluginApiVersionUseCasePort, GetPluginDtoUseCase,
        GetPluginDtoUseCasePort, GetPluginSettingsUseCase, GetPluginSettingsUseCasePort,
        ImportPluginsUseCase, ImportPluginsUseCasePort, ListPluginsDtoUseCase,
        ListPluginsDtoUseCasePort, LoadConfigType, PluginDisableService, PluginExtractor,
        PluginLoader, PluginLoaderRegistry, PluginProvider, PluginProviderFactory, PluginRegistry,
        PluginSettingsStorage, PluginSourceStorage, PluginStorage, PluginSyncService,
        PluginsFeature, RemovePluginUseCase, RemovePluginUseCasePort, SyncPluginsUseCase,
        UpdatePluginUseCase, UpdatePluginUseCasePort,
        infra::{
            ExtismPluginLoader, FsPluginSettingsStorage, FsPluginSourceStorage, FsPluginStorage,
            GithubProvider, PluginInfrastructureListener, ZipPluginExtractor,
        },
    },
    process::{
        GetProcessMetadataByInstanceIdUseCase, GetProcessMetadataByInstanceIdUseCasePort,
        KillProcessUseCase, KillProcessUseCasePort, ListProcessMetadataUseCase,
        ListProcessMetadataUseCasePort, ManageProcessService, ManageProcessUseCase, ProcessFeature,
        ProcessStartService, ProcessStorage, StartProcessUseCase, TrackProcessService,
        TrackProcessUseCase, WaitForProcessUseCase, WaitForProcessUseCasePort,
        infra::InMemoryProcessStorage,
    },
    settings::{
        self, DefaultInstanceSettingsStorage, EditDefaultInstanceSettingsUseCase,
        EditDefaultInstanceSettingsUseCasePort, EditSettingsUseCase, EditSettingsUseCasePort,
        GetDefaultInstanceSettingsUseCase, GetDefaultInstanceSettingsUseCasePort,
        GetSettingsUseCase, GetSettingsUseCasePort, LocationInfo, Settings, SettingsFeature,
        SettingsStorage,
        infra::{
            FsDefaultInstanceSettingsStorage, FsSettingsStorage,
            SqliteDefaultInstanceSettingsStorage, SqliteSettingsStorage,
        },
    },
};
use crate::shared::{
    cache::{
        domain::AssetsStorage,
        infra::{FileCache, FsAssetsStorage, SqliteCache},
    },
    capability::{domain::CapabilityRegistry, infra::MemoryCapabilityRegistry},
    fetch::domain::FetchSemaphore,
    request_client::infra::ReqwestClient,
};

type PBarStorage = InMemoryProgressBarStorage;
type ProgressServiceType = ProgressServiceImpl<PBarStorage>;

struct UseCaseCache {
    start_process_uc: OnceLock<Arc<dyn ProcessStartService>>,
    track_process_uc: OnceLock<Arc<dyn TrackProcessService>>,
    manage_process_uc: OnceLock<Arc<dyn ManageProcessService>>,
    install_instance_uc: OnceLock<Arc<dyn InstanceInstallService>>,
    disable_plugin_uc: OnceLock<Arc<dyn PluginDisableService>>,
    sync_plugins_uc: OnceLock<Arc<dyn PluginSyncService>>,
    health_service: OnceLock<Arc<dyn MinecraftHealthService>>,
    launch_command_service: OnceLock<Arc<dyn MinecraftLaunchCommandService>>,
    launch_instance_uc: OnceLock<Arc<dyn InstanceLaunchService>>,
    minecraft_install_uc: OnceLock<Arc<dyn MinecraftInstallService>>,
}

impl UseCaseCache {
    fn new() -> Self {
        Self {
            start_process_uc: OnceLock::new(),
            track_process_uc: OnceLock::new(),
            manage_process_uc: OnceLock::new(),
            install_instance_uc: OnceLock::new(),
            disable_plugin_uc: OnceLock::new(),
            sync_plugins_uc: OnceLock::new(),
            health_service: OnceLock::new(),
            launch_command_service: OnceLock::new(),
            launch_instance_uc: OnceLock::new(),
            minecraft_install_uc: OnceLock::new(),
        }
    }
}

static AETHER_CONTAINER: OnceLock<Arc<AetherContainer>> = OnceLock::new();

pub struct AetherContainer {
    cache: UseCaseCache,
    credentials_storage: Arc<dyn CredentialsStorage>,
    settings_storage: Arc<dyn SettingsStorage>,
    default_instance_settings_storage: Arc<dyn DefaultInstanceSettingsStorage>,
    process_storage: Arc<dyn ProcessStorage>,
    instance_storage: Arc<dyn InstanceStorage>,
    pack_storage: Arc<dyn PackStorage>,
    content_file_service: Arc<dyn ContentFileService>,
    instance_file_service: Arc<dyn InstanceFileService>,
    java_storage: Arc<dyn JavaStorage>,
    java_installation_service: Arc<dyn JavaInstallationService>,
    java_installation_tracker: Arc<dyn JavaInstallationTracker>,
    jre_provider: Arc<dyn JreProvider>,
    metadata_storage: Arc<dyn MetadataStorage>,
    #[allow(dead_code)]
    minecraft_cache: Arc<FileCache<crate::features::minecraft::infra::MinecraftDownloadResolver>>,
    minecraft_downloader: Arc<dyn MinecraftDownloader>,
    forge_processor: Arc<dyn ModLoaderProcessor>,
    plugin_registry: Arc<PluginRegistry>,
    plugin_loader_registry: Arc<PluginLoaderRegistry>,
    plugin_storage: Arc<dyn PluginStorage>,
    plugin_source_storage: Arc<dyn PluginSourceStorage>,
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
    plugin_provider_factory: Arc<PluginProviderFactory>,
    plugin_extractor: Arc<dyn PluginExtractor>,
    event_emitter: SharedEventEmitter,
    progress_bar_storage: Arc<PBarStorage>,
    progress_service: Arc<ProgressServiceType>,
    location_info: Arc<LocationInfo>,
    #[allow(dead_code)]
    request_client: Arc<ReqwestClient<ProgressServiceType>>,
    instance_watcher_service: Arc<dyn InstanceWatcherService>,
    importers_registry: Arc<dyn CapabilityRegistry<Arc<dyn Importer>>>,
    updaters_registry: Arc<dyn CapabilityRegistry<Arc<dyn Updater>>>,
    content_provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>>,
    assets_storage: Arc<dyn AssetsStorage>,
}

impl AetherContainer {
    #[allow(clippy::similar_names, clippy::too_many_lines)]
    pub async fn init(
        location_info: Arc<LocationInfo>,
        event_emitter: SharedEventEmitter,
        sqlite_pool: sqlx::SqlitePool,
    ) -> crate::Result<()> {
        let config_dir = location_info.config_dir();
        let migrated_dir_name = "migrated";

        settings::infra::migrate_settings_to_sqlite(
            &FsSettingsStorage::new(config_dir),
            &SqliteSettingsStorage::new(sqlite_pool.clone()),
            migrated_dir_name,
        )
        .await?;

        settings::infra::migrate_default_instance_settings_to_sqlite(
            &FsDefaultInstanceSettingsStorage::new(config_dir),
            &SqliteDefaultInstanceSettingsStorage::new(sqlite_pool.clone()),
            migrated_dir_name,
        )
        .await?;

        auth::infra::migrate_credentials_to_sqlite(
            &FsCredentialsStorage::new(config_dir),
            &SqliteCredentialsStorage::new(sqlite_pool.clone()),
            migrated_dir_name,
        )
        .await?;

        instance::infra::migrate_instances_to_sqlite(
            &crate::features::instance::infra::FsInstanceStorage::new(location_info.clone()),
            &SqliteInstanceStorage::new(sqlite_pool.clone()),
        )
        .await?;

        instance::infra::migrate_packs_to_sqlite(
            &crate::features::instance::infra::FsInstanceStorage::new(location_info.clone()),
            &crate::features::instance::infra::FsPackStorage::new(location_info.clone()),
            &SqlitePackStorage::new(sqlite_pool.clone()),
        )
        .await?;

        java::infra::migrate_java_to_sqlite(
            &location_info.java_dir(),
            &FsJavaInstallationService,
            &SqliteJavaStorage::new(sqlite_pool.clone()),
        )
        .await?;

        minecraft::infra::migrate_minecraft_metadata_to_sqlite(&location_info).await;

        let settings_storage_sqlite = Arc::new(SqliteSettingsStorage::new(sqlite_pool.clone()));
        let settings = if let Ok(s) = settings_storage_sqlite.get().await {
            s
        } else {
            let default = Settings::default();
            let _ = settings_storage_sqlite.upsert(default.clone()).await;
            default
        };
        let max_downloads = settings.max_concurrent_downloads();
        let fetch_semaphore = Arc::new(FetchSemaphore(tokio::sync::Semaphore::new(max_downloads)));

        let reqwest_client: Arc<ClientWithMiddleware> = {
            let client = reqwest::Client::builder()
                .tcp_keepalive(Some(std::time::Duration::from_secs(10)))
                .build()
                .expect("Failed to build reqwest client");
            let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);
            let retry_middleware =
                reqwest_retry::RetryTransientMiddleware::new_with_policy(retry_policy);
            Arc::new(
                reqwest_middleware::ClientBuilder::new(client)
                    .with(retry_middleware)
                    .build(),
            )
        };

        let progress_bar_storage = Arc::new(InMemoryProgressBarStorage::default());
        let progress_service: Arc<ProgressServiceType> = Arc::new(ProgressServiceImpl::new(
            event_emitter.clone(),
            progress_bar_storage.clone(),
        ));
        let http_client: Arc<ReqwestClient<ProgressServiceType>> = Arc::new(ReqwestClient::new(
            progress_service.clone(),
            reqwest_client.clone(),
            fetch_semaphore,
        ));

        let credentials_storage: Arc<dyn CredentialsStorage> =
            Arc::new(SqliteCredentialsStorage::new(sqlite_pool.clone()));

        let settings_storage: Arc<dyn SettingsStorage> = settings_storage_sqlite.clone();
        let default_instance_settings_storage: Arc<dyn DefaultInstanceSettingsStorage> = Arc::new(
            SqliteDefaultInstanceSettingsStorage::new(sqlite_pool.clone()),
        );

        let process_storage: Arc<dyn ProcessStorage> = Arc::new(InMemoryProcessStorage::default());

        let instance_storage: Arc<dyn InstanceStorage> =
            Arc::new(EventEmittingInstanceStorage::new(
                event_emitter.clone(),
                SqliteInstanceStorage::new(sqlite_pool.clone()),
            ));
        let pack_storage: Arc<dyn PackStorage> =
            Arc::new(SqlitePackStorage::new(sqlite_pool.clone()));
        let content_file_service: Arc<dyn ContentFileService> =
            Arc::new(FsContentFileService::new(location_info.clone()));
        let instance_file_service: Arc<dyn InstanceFileService> =
            Arc::new(FsInstanceFileService::new(location_info.clone()));

        let java_storage: Arc<dyn JavaStorage> =
            Arc::new(SqliteJavaStorage::new(sqlite_pool.clone()));
        let java_installation_service: Arc<dyn JavaInstallationService> =
            Arc::new(FsJavaInstallationService);
        let java_installation_tracker: Arc<dyn JavaInstallationTracker> =
            Arc::new(MemoryJavaInstallationTracker::default());
        let jre_provider: Arc<dyn JreProvider> = Arc::new(AzulJreProvider::new(
            progress_service.clone(),
            http_client.clone(),
        ));

        let metadata_storage: Arc<dyn MetadataStorage> = Arc::new(CachedMetadataStorage::new(
            SqliteCache::new(sqlite_pool.clone()),
            ModrinthMetadataStorage::new(http_client.clone()),
        ));

        let minecraft_cache = Arc::new(FileCache::new(MinecraftDownloadResolver::new(
            location_info.clone(),
        )));

        let client_svc = ClientService::new(
            progress_service.clone(),
            http_client.clone(),
            minecraft_cache.clone(),
        );
        let assets_svc = AssetsService::new(
            progress_service.clone(),
            http_client.clone(),
            location_info.clone(),
            minecraft_cache.clone(),
        );
        let libraries_svc = LibrariesService::new(
            progress_service.clone(),
            http_client.clone(),
            location_info.clone(),
        );
        let minecraft_downloader: Arc<dyn MinecraftDownloader> =
            Arc::new(MinecraftDownloadService::new(
                client_svc,
                assets_svc,
                libraries_svc,
                http_client.clone(),
                progress_service.clone(),
                minecraft_cache.clone(),
            ));
        let forge_processor: Arc<dyn ModLoaderProcessor> = Arc::new(ForgeProcessor::new(
            progress_service.clone(),
            location_info.clone(),
        ));

        let plugin_registry = Arc::new(PluginRegistry::new(event_emitter.clone()));
        let plugin_loader_registry = Arc::new(PluginLoaderRegistry::new(HashMap::from([(
            LoadConfigType::Extism,
            Arc::new(ExtismPluginLoader::new(location_info.clone())) as Arc<dyn PluginLoader>,
        )])));
        let plugin_storage: Arc<dyn PluginStorage> =
            Arc::new(FsPluginStorage::new(location_info.clone(), None));
        let plugin_source_storage: Arc<dyn PluginSourceStorage> =
            Arc::new(FsPluginSourceStorage::new(location_info.clone()));
        let plugin_settings_storage: Arc<dyn PluginSettingsStorage> =
            Arc::new(FsPluginSettingsStorage::new(location_info.clone()));
        let plugin_provider_factory =
            Arc::new(PluginProviderFactory::new(vec![
                Box::new(GithubProvider::new(reqwest_client.clone())) as Box<dyn PluginProvider>,
            ]));
        let plugin_extractor: Arc<dyn PluginExtractor> = Arc::new(ZipPluginExtractor::default());

        let instance_watcher_service: Arc<dyn InstanceWatcherService> = {
            let event_handler = Arc::new(InstanceEventHandler::new(event_emitter.clone()));
            let watcher = NotifyFileWatcher::new(event_handler).map_err(|e| {
                crate::ErrorKind::CoreError(format!("File watcher: {e}")).as_error()
            })?;
            Arc::new(InstanceWatcherServiceImpl::new(
                Arc::new(watcher),
                location_info.clone(),
            ))
        };

        let importers_registry: Arc<dyn CapabilityRegistry<Arc<dyn Importer>>> =
            Arc::new(MemoryCapabilityRegistry::new("importer"));
        let updaters_registry: Arc<dyn CapabilityRegistry<Arc<dyn Updater>>> =
            Arc::new(MemoryCapabilityRegistry::new("updater"));

        let content_provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>> = {
            let registry: Arc<MemoryCapabilityRegistry<Arc<dyn ContentProvider>>> =
                Arc::new(MemoryCapabilityRegistry::new("content_provider"));
            let provider = Arc::new(ModrinthContentProvider::new(
                location_info.clone(),
                None,
                http_client.clone(),
            ));
            let meta = provider.metadata();
            let _ = registry
                .add(
                    ModrinthContentProvider::ID.to_owned(),
                    meta.id.clone(),
                    provider,
                )
                .await;
            registry
        };

        let file_cache_assets = Arc::new(FileCache::new(
            crate::shared::cache::infra::AssetsResolver::new(location_info.clone()),
        ));
        let assets_storage: Arc<dyn AssetsStorage> =
            Arc::new(FsAssetsStorage::new(file_cache_assets));

        let container = Arc::new(Self {
            cache: UseCaseCache::new(),
            credentials_storage,
            settings_storage,
            default_instance_settings_storage,
            process_storage,
            instance_storage,
            pack_storage,
            content_file_service,
            instance_file_service,
            java_storage,
            java_installation_service,
            java_installation_tracker,
            jre_provider,
            metadata_storage,
            minecraft_cache,
            minecraft_downloader,
            forge_processor,
            plugin_registry,
            plugin_loader_registry,
            plugin_storage,
            plugin_source_storage,
            plugin_settings_storage,
            plugin_provider_factory,
            plugin_extractor,
            event_emitter: event_emitter.clone(),
            progress_bar_storage,
            progress_service,
            location_info,
            request_client: http_client,
            instance_watcher_service,
            importers_registry,
            updaters_registry,
            content_provider_registry,
            assets_storage,
        });

        AETHER_CONTAINER.set(container).map_err(|_| {
            crate::ErrorKind::CoreError("AetherContainer already initialised".into()).as_error()
        })?;

        let c = Self::get();
        c.instance_watcher_service.watch_instances().await?;

        let plugin_infra_listener = Arc::new(PluginInfrastructureListener::new(
            c.plugin_registry.clone(),
            c.importers_registry.clone(),
            c.updaters_registry.clone(),
            c.content_provider_registry.clone(),
        ));
        c.event_emitter.on::<PluginEvent, _>({
            let listener = plugin_infra_listener.clone();
            move |event| {
                let task = listener.clone();
                tokio::spawn(async move { task.on_plugin_event(event).await });
            }
        });

        log::info!("AetherContainer initialized");
        Ok(())
    }

    pub fn get() -> Arc<Self> {
        AETHER_CONTAINER
            .get()
            .expect("AetherContainer not initialised")
            .clone()
    }

    pub fn try_get() -> Option<Arc<Self>> {
        AETHER_CONTAINER.get().map(Arc::clone)
    }

    pub fn progress_bar_storage(&self) -> Arc<PBarStorage> {
        self.progress_bar_storage.clone()
    }

    pub fn event_emitter(&self) -> SharedEventEmitter {
        self.event_emitter.clone()
    }

    fn loader_version_resolver(&self) -> Arc<dyn LoaderVersionService> {
        Arc::new(LoaderVersionResolver::new(self.metadata_storage.clone()))
    }

    fn get_version_manifest_use_case(&self) -> Arc<dyn VersionManifestService> {
        Arc::new(GetVersionManifestUseCase::new(
            self.metadata_storage.clone(),
        ))
    }

    fn get_java_use_case(&self) -> Arc<dyn JavaQueryService> {
        Arc::new(GetJavaUseCase::new(
            self.java_storage.clone(),
            self.java_installation_service.clone(),
        ))
    }

    fn install_java_use_case(&self) -> Arc<dyn JavaInstallService> {
        Arc::new(InstallJavaUseCase::new(
            self.java_storage.clone(),
            self.java_installation_service.clone(),
            self.jre_provider.clone(),
            self.location_info.clone(),
            self.java_installation_tracker.clone(),
        ))
    }

    fn minecraft_install_use_case(&self) -> Arc<dyn MinecraftInstallService> {
        self.cache
            .minecraft_install_uc
            .get_or_init(|| {
                Arc::new(InstallMinecraftUseCase::new(
                    self.loader_version_resolver(),
                    self.get_version_manifest_use_case(),
                    self.minecraft_downloader.clone(),
                    self.forge_processor.clone(),
                    self.java_installation_service.clone(),
                    self.get_java_use_case(),
                    self.install_java_use_case(),
                ))
            })
            .clone()
    }

    fn instance_install_service(&self) -> Arc<dyn InstanceInstallService> {
        self.cache
            .install_instance_uc
            .get_or_init(|| {
                Arc::new(InstallInstanceUseCase::new(
                    self.instance_storage.clone(),
                    self.minecraft_install_use_case(),
                    self.progress_service.clone(),
                    self.location_info.clone(),
                ))
            })
            .clone()
    }

    fn health_service(&self) -> Arc<dyn MinecraftHealthService> {
        self.cache
            .health_service
            .get_or_init(|| {
                Arc::new(MinecraftFileHealthService::new(
                    self.loader_version_resolver(),
                    self.get_version_manifest_use_case(),
                    self.minecraft_downloader.clone(),
                    self.get_java_use_case(),
                    self.location_info.clone(),
                ))
            })
            .clone()
    }

    fn launch_command_service(&self) -> Arc<dyn MinecraftLaunchCommandService> {
        self.cache
            .launch_command_service
            .get_or_init(|| {
                Arc::new(GetMinecraftLaunchCommandUseCase::new(
                    self.loader_version_resolver(),
                    self.get_version_manifest_use_case(),
                    self.minecraft_downloader.clone(),
                    self.java_installation_service.clone(),
                    self.get_java_use_case(),
                    self.install_java_use_case(),
                    self.location_info.clone(),
                ))
            })
            .clone()
    }

    fn launch_instance_use_case(&self) -> Arc<dyn InstanceLaunchService> {
        self.cache
            .launch_instance_uc
            .get_or_init(|| {
                let manage = self.manage_process_service();
                let start: Arc<dyn ProcessStartService> = Arc::new(StartProcessUseCase::new(
                    self.event_emitter.clone(),
                    self.process_storage.clone(),
                    manage,
                ));
                Arc::new(LaunchInstanceUseCase::new(
                    self.plugin_registry.clone(),
                    self.instance_storage.clone(),
                    self.default_instance_settings_storage.clone(),
                    self.location_info.clone(),
                    self.process_storage.clone(),
                    self.instance_install_service(),
                    self.health_service(),
                    self.launch_command_service(),
                    start,
                ))
            })
            .clone()
    }

    fn track_process_service(&self) -> Arc<dyn TrackProcessService> {
        self.cache
            .track_process_uc
            .get_or_init(|| {
                Arc::new(TrackProcessUseCase::new(
                    self.process_storage.clone(),
                    self.instance_storage.clone(),
                ))
            })
            .clone()
    }

    fn manage_process_service(&self) -> Arc<dyn ManageProcessService> {
        self.cache
            .manage_process_uc
            .get_or_init(|| {
                Arc::new(ManageProcessUseCase::new(
                    self.event_emitter.clone(),
                    self.process_storage.clone(),
                    self.track_process_service(),
                    self.location_info.clone(),
                ))
            })
            .clone()
    }

    fn process_start_service(&self) -> Arc<dyn ProcessStartService> {
        self.cache
            .start_process_uc
            .get_or_init(|| {
                Arc::new(StartProcessUseCase::new(
                    self.event_emitter.clone(),
                    self.process_storage.clone(),
                    self.manage_process_service(),
                ))
            })
            .clone()
    }

    fn disable_plugin_service(&self) -> Arc<dyn PluginDisableService> {
        self.cache
            .disable_plugin_uc
            .get_or_init(|| {
                Arc::new(DisablePluginUseCase::new(
                    self.plugin_registry.clone(),
                    self.plugin_loader_registry.clone(),
                    self.settings_storage.clone(),
                ))
            })
            .clone()
    }

    fn sync_plugins_service(&self) -> Arc<dyn PluginSyncService> {
        self.cache
            .sync_plugins_uc
            .get_or_init(|| {
                Arc::new(SyncPluginsUseCase::new(
                    self.plugin_storage.clone(),
                    self.plugin_registry.clone(),
                    self.disable_plugin_service(),
                    self.event_emitter.clone(),
                ))
            })
            .clone()
    }
}

// Feature trait implementations

impl SettingsFeature for AetherContainer {
    fn get_settings_use_case(&self) -> Arc<dyn GetSettingsUseCasePort> {
        Arc::new(GetSettingsUseCase::new(self.settings_storage.clone()))
    }
    fn get_default_instance_settings_use_case(
        &self,
    ) -> Arc<dyn GetDefaultInstanceSettingsUseCasePort> {
        Arc::new(GetDefaultInstanceSettingsUseCase::new(
            self.default_instance_settings_storage.clone(),
        ))
    }
    fn edit_settings_use_case(&self) -> Arc<dyn EditSettingsUseCasePort> {
        Arc::new(EditSettingsUseCase::new(self.settings_storage.clone()))
    }
    fn edit_default_instance_settings_use_case(
        &self,
    ) -> Arc<dyn EditDefaultInstanceSettingsUseCasePort> {
        Arc::new(EditDefaultInstanceSettingsUseCase::new(
            self.default_instance_settings_storage.clone(),
        ))
    }
    fn settings_storage(&self) -> Arc<dyn SettingsStorage> {
        self.settings_storage.clone()
    }
    fn default_instance_settings_storage(&self) -> Arc<dyn DefaultInstanceSettingsStorage> {
        self.default_instance_settings_storage.clone()
    }
    fn location_info(&self) -> Arc<LocationInfo> {
        self.location_info.clone()
    }
}

impl AuthFeature for AetherContainer {
    fn create_offline_account_use_case(&self) -> Arc<dyn CreateOfflineAccountUseCasePort> {
        Arc::new(CreateOfflineAccountUseCase::new(
            self.credentials_storage.clone(),
        ))
    }
    fn get_accounts_use_case(&self) -> Arc<dyn GetAccountsUseCasePort> {
        Arc::new(GetAccountsUseCase::new(self.credentials_storage.clone()))
    }
    fn logout_use_case(&self) -> Arc<dyn LogoutUseCasePort> {
        Arc::new(LogoutUseCase::new(self.credentials_storage.clone()))
    }
    fn set_active_account_use_case(&self) -> Arc<dyn SetActiveAccountUseCasePort> {
        Arc::new(SetActiveAccountUseCase::new(
            self.credentials_storage.clone(),
        ))
    }
    fn active_account_helper(&self) -> Arc<ActiveAccountHelper> {
        Arc::new(ActiveAccountHelper)
    }
}

impl JavaFeature for AetherContainer {
    fn discover_java_use_case(&self) -> Arc<dyn DiscoverJavaUseCasePort> {
        Arc::new(DiscoverJavaUseCase::new(
            self.java_installation_service.clone(),
            get_default_discovery_paths(),
        ))
    }
    fn edit_java_use_case(&self) -> Arc<dyn EditJavaUseCasePort> {
        Arc::new(EditJavaUseCase::new(
            self.java_storage.clone(),
            self.java_installation_service.clone(),
        ))
    }
    fn get_active_java_installations_use_case(
        &self,
    ) -> Arc<dyn GetActiveJavaInstallationsUseCasePort> {
        Arc::new(GetActiveJavaInstallationsUseCase::new(
            self.java_installation_tracker.clone(),
        ))
    }
    fn list_java_use_case(&self) -> Arc<dyn ListJavaUseCasePort> {
        Arc::new(ListJavaUseCase::new(self.java_storage.clone()))
    }
    fn remove_java_use_case(&self) -> Arc<dyn RemoveJavaUseCasePort> {
        Arc::new(RemoveJavaUseCase::new(self.java_storage.clone()))
    }
    fn test_jre_use_case(&self) -> Arc<dyn TestJreUseCasePort> {
        Arc::new(TestJreUseCase::new(self.java_installation_service.clone()))
    }
    fn get_java_use_case(&self) -> Arc<dyn JavaQueryService> {
        self.get_java_use_case()
    }
    fn install_java_use_case(&self) -> Arc<dyn JavaInstallService> {
        self.install_java_use_case()
    }
    fn java_installation_service(&self) -> Arc<dyn JavaInstallationService> {
        self.java_installation_service.clone()
    }
    fn java_installation_tracker(&self) -> Arc<dyn JavaInstallationTracker> {
        self.java_installation_tracker.clone()
    }
    fn java_storage(&self) -> Arc<dyn JavaStorage> {
        self.java_storage.clone()
    }
    fn jre_provider(&self) -> Arc<dyn JreProvider> {
        self.jre_provider.clone()
    }
}

impl MinecraftFeature for AetherContainer {
    fn get_version_manifest_use_case(&self) -> Arc<dyn VersionManifestService> {
        self.get_version_manifest_use_case()
    }
    fn install_minecraft_use_case(&self) -> Arc<dyn MinecraftInstallService> {
        self.minecraft_install_use_case()
    }
    fn get_minecraft_launch_command_use_case(&self) -> Arc<dyn MinecraftLaunchCommandService> {
        self.launch_command_service()
    }
    fn get_loader_version_manifest_use_case(&self) -> Arc<dyn GetLoaderVersionManifestUseCasePort> {
        Arc::new(GetLoaderVersionManifestUseCase::new(
            self.metadata_storage.clone(),
        ))
    }
    fn loader_version_service(&self) -> Arc<dyn LoaderVersionService> {
        self.loader_version_resolver()
    }
    fn metadata_storage(&self) -> Arc<dyn MetadataStorage> {
        self.metadata_storage.clone()
    }
    fn minecraft_downloader(&self) -> Arc<dyn MinecraftDownloader> {
        self.minecraft_downloader.clone()
    }
    fn minecraft_health_service(&self) -> Arc<dyn MinecraftHealthService> {
        self.health_service()
    }
    fn mod_loader_processor(&self) -> Arc<dyn ModLoaderProcessor> {
        self.forge_processor.clone()
    }
}

impl InstanceFeature for AetherContainer {
    fn create_instance_use_case(&self) -> Arc<dyn CreateInstanceUseCasePort> {
        Arc::new(CreateInstanceUseCase::new(
            self.instance_storage.clone(),
            self.loader_version_resolver(),
            self.instance_install_service(),
            self.location_info.clone(),
            self.event_emitter.clone(),
            self.instance_watcher_service.clone(),
            self.instance_file_service.clone(),
        ))
    }
    fn get_instance_use_case(&self) -> Arc<dyn GetInstanceUseCasePort> {
        Arc::new(GetInstanceUseCase::new(self.instance_storage.clone()))
    }
    fn list_instances_use_case(&self) -> Arc<dyn ListInstancesUseCasePort> {
        Arc::new(ListInstancesUseCase::new(self.instance_storage.clone()))
    }
    fn edit_instance_use_case(&self) -> Arc<dyn EditInstanceUseCasePort> {
        Arc::new(EditInstanceUseCase::new(self.instance_storage.clone()))
    }
    fn edit_instance_icon_use_case(&self) -> Arc<dyn EditInstanceIconUseCasePort> {
        Arc::new(EditInstanceIconUseCase::new(
            self.instance_storage.clone(),
            self.assets_storage.clone(),
        ))
    }
    fn remove_instance_use_case(&self) -> Arc<dyn RemoveInstanceUseCasePort> {
        Arc::new(RemoveInstanceUseCase::new(
            self.instance_storage.clone(),
            self.instance_watcher_service.clone(),
            self.instance_file_service.clone(),
            self.pack_storage.clone(),
        ))
    }
    fn update_instance_use_case(&self) -> Arc<dyn UpdateInstanceUseCasePort> {
        Arc::new(UpdateInstanceUseCase::new(
            self.instance_storage.clone(),
            self.updaters_registry.clone(),
        ))
    }
    fn import_instance_use_case(&self) -> Arc<dyn ImportInstanceUseCasePort> {
        Arc::new(ImportInstanceUseCase::new(self.importers_registry.clone()))
    }
    fn list_importers_use_case(&self) -> Arc<dyn ListImportersUseCasePort> {
        Arc::new(ListImportersUseCase::new(self.importers_registry.clone()))
    }
    fn launch_instance_with_active_account_use_case(
        &self,
    ) -> Arc<dyn LaunchInstanceWithActiveAccountUseCasePort> {
        Arc::new(LaunchInstanceWithActiveAccountUseCase::new(
            self.credentials_storage.clone(),
            self.launch_instance_use_case(),
        ))
    }
    fn change_content_state_use_case(&self) -> Arc<dyn ChangeContentStateUseCasePort> {
        Arc::new(ChangeContentStateUseCase::new(
            self.event_emitter.clone(),
            self.content_file_service.clone(),
        ))
    }
    fn import_content_use_case(&self) -> Arc<dyn ImportContentUseCasePort> {
        Arc::new(ImportContentUseCase::new(
            self.event_emitter.clone(),
            self.pack_storage.clone(),
            self.location_info.clone(),
        ))
    }
    fn list_content_use_case(&self) -> Arc<dyn ListContentUseCasePort> {
        Arc::new(ListContentUseCase::new(
            self.pack_storage.clone(),
            self.location_info.clone(),
        ))
    }
    fn remove_content_use_case(&self) -> Arc<dyn RemoveContentUseCasePort> {
        Arc::new(RemoveContentUseCase::new(
            self.event_emitter.clone(),
            self.pack_storage.clone(),
            self.content_file_service.clone(),
        ))
    }
    fn search_content_use_case(&self) -> Arc<dyn SearchContentUseCasePort> {
        Arc::new(SearchContentUseCase::new(
            self.content_provider_registry.clone(),
        ))
    }
    fn get_content_use_case(&self) -> Arc<dyn GetContentUseCasePort> {
        Arc::new(GetContentUseCase::new(
            self.content_provider_registry.clone(),
        ))
    }
    fn install_content_use_case(&self) -> Arc<dyn InstallContentUseCasePort> {
        Arc::new(InstallContentUseCase::new(
            self.pack_storage.clone(),
            self.content_provider_registry.clone(),
            self.content_file_service.clone(),
        ))
    }
    fn check_content_compatibility_use_case(
        &self,
    ) -> Arc<dyn CheckContentCompatibilityUseCasePort> {
        Arc::new(CheckContentCompatibilityUseCase::new(
            self.content_provider_registry.clone(),
            self.instance_storage.clone(),
        ))
    }
    fn list_content_versions_use_case(&self) -> Arc<dyn ListContentVersionsUseCasePort> {
        Arc::new(ListContentVersionsUseCase::new(
            self.content_provider_registry.clone(),
        ))
    }
    fn list_providers_use_case(&self) -> Arc<dyn ListProvidersUseCasePort> {
        Arc::new(ListProvidersUseCase::new(
            self.content_provider_registry.clone(),
        ))
    }
    fn instance_storage(&self) -> Arc<dyn InstanceStorage> {
        self.instance_storage.clone()
    }
    fn pack_storage(&self) -> Arc<dyn PackStorage> {
        self.pack_storage.clone()
    }
    fn instance_install_service(&self) -> Arc<dyn InstanceInstallService> {
        self.instance_install_service()
    }
    fn instance_launch_service(&self) -> Arc<dyn InstanceLaunchService> {
        self.launch_instance_use_case()
    }
    fn instance_watcher_service(&self) -> Arc<dyn InstanceWatcherService> {
        self.instance_watcher_service.clone()
    }
    fn instance_file_service(&self) -> Arc<dyn InstanceFileService> {
        self.instance_file_service.clone()
    }
    fn content_file_service(&self) -> Arc<dyn ContentFileService> {
        self.content_file_service.clone()
    }
}

impl ProcessFeature for AetherContainer {
    fn wait_for_process_use_case(&self) -> Arc<dyn WaitForProcessUseCasePort> {
        Arc::new(WaitForProcessUseCase::new(self.process_storage.clone()))
    }
    fn kill_process_use_case(&self) -> Arc<dyn KillProcessUseCasePort> {
        Arc::new(KillProcessUseCase::new(self.process_storage.clone()))
    }
    fn list_process_metadata_use_case(&self) -> Arc<dyn ListProcessMetadataUseCasePort> {
        Arc::new(ListProcessMetadataUseCase::new(
            self.process_storage.clone(),
        ))
    }
    fn get_process_metadata_by_instance_id_use_case(
        &self,
    ) -> Arc<dyn GetProcessMetadataByInstanceIdUseCasePort> {
        Arc::new(GetProcessMetadataByInstanceIdUseCase::new(
            self.process_storage.clone(),
        ))
    }
    fn process_start_service(&self) -> Arc<dyn ProcessStartService> {
        self.process_start_service()
    }
    fn track_process_service(&self) -> Arc<dyn TrackProcessService> {
        self.track_process_service()
    }
    fn manage_process_service(&self) -> Arc<dyn ManageProcessService> {
        self.manage_process_service()
    }
    fn process_storage(&self) -> Arc<dyn ProcessStorage> {
        self.process_storage.clone()
    }
}

impl PluginsFeature for AetherContainer {
    fn check_for_plugin_updates_use_case(&self) -> Arc<dyn CheckForPluginUpdatesUseCasePort> {
        Arc::new(CheckForPluginUpdatesUseCase::new(
            self.plugin_source_storage.clone(),
            self.plugin_provider_factory.clone(),
        ))
    }
    fn edit_plugin_settings_use_case(&self) -> Arc<dyn EditPluginSettingsUseCasePort> {
        Arc::new(EditPluginSettingsUseCase::new(
            self.plugin_settings_storage.clone(),
        ))
    }
    fn enable_plugin_use_case(&self) -> Arc<dyn EnablePluginUseCasePort> {
        Arc::new(EnablePluginUseCase::new(
            self.plugin_registry.clone(),
            self.plugin_loader_registry.clone(),
            self.plugin_settings_storage.clone(),
            self.settings_storage.clone(),
        ))
    }
    fn force_enable_plugin_use_case(&self) -> Arc<dyn ForceEnablePluginUseCasePort> {
        Arc::new(ForceEnablePluginUseCase::new(
            self.plugin_registry.clone(),
            self.plugin_loader_registry.clone(),
            self.plugin_settings_storage.clone(),
            self.settings_storage.clone(),
        ))
    }
    fn get_plugin_api_version_use_case(&self) -> Arc<dyn GetPluginApiVersionUseCasePort> {
        Arc::new(GetPluginApiVersionUseCase::default())
    }
    fn get_plugin_dto_use_case(&self) -> Arc<dyn GetPluginDtoUseCasePort> {
        Arc::new(GetPluginDtoUseCase::new(self.plugin_registry.clone()))
    }
    fn get_plugin_settings_use_case(&self) -> Arc<dyn GetPluginSettingsUseCasePort> {
        Arc::new(GetPluginSettingsUseCase::new(
            self.plugin_settings_storage.clone(),
        ))
    }
    fn import_plugins_use_case(&self) -> Arc<dyn ImportPluginsUseCasePort> {
        Arc::new(ImportPluginsUseCase::new(
            self.plugin_extractor.clone(),
            self.plugin_storage.clone(),
            self.sync_plugins_service(),
        ))
    }
    fn list_plugins_dto_use_case(&self) -> Arc<dyn ListPluginsDtoUseCasePort> {
        Arc::new(ListPluginsDtoUseCase::new(self.plugin_registry.clone()))
    }
    fn remove_plugin_use_case(&self) -> Arc<dyn RemovePluginUseCasePort> {
        Arc::new(RemovePluginUseCase::new(
            self.plugin_storage.clone(),
            self.sync_plugins_service(),
        ))
    }
    fn update_plugin_use_case(&self) -> Arc<dyn UpdatePluginUseCasePort> {
        Arc::new(UpdatePluginUseCase::new(
            self.plugin_extractor.clone(),
            self.plugin_storage.clone(),
            self.plugin_source_storage.clone(),
            self.plugin_provider_factory.clone(),
        ))
    }
    fn plugin_registry(&self) -> Arc<PluginRegistry> {
        self.plugin_registry.clone()
    }
    fn plugin_loader_registry(&self) -> Arc<PluginLoaderRegistry> {
        self.plugin_loader_registry.clone()
    }
    fn plugin_provider_factory(&self) -> Arc<PluginProviderFactory> {
        self.plugin_provider_factory.clone()
    }
    fn plugin_source_storage(&self) -> Arc<dyn PluginSourceStorage> {
        self.plugin_source_storage.clone()
    }
    fn plugin_loader(&self) -> Arc<dyn PluginLoader> {
        self.plugin_loader_registry
            .get(&LoadConfigType::Extism)
            .expect("ExtismPluginLoader must be registered")
            .clone()
    }
    fn plugin_storage(&self) -> Arc<dyn PluginStorage> {
        self.plugin_storage.clone()
    }
    fn plugin_settings_storage(&self) -> Arc<dyn PluginSettingsStorage> {
        self.plugin_settings_storage.clone()
    }
    fn plugin_extractor(&self) -> Arc<dyn PluginExtractor> {
        self.plugin_extractor.clone()
    }
    fn sync_plugins_use_case(&self) -> Arc<dyn PluginSyncService> {
        self.sync_plugins_service()
    }
    fn plugin_sync_service(&self) -> Arc<dyn PluginSyncService> {
        self.sync_plugins_service()
    }
    fn disable_plugin_use_case(&self) -> Arc<dyn PluginDisableService> {
        self.disable_plugin_service()
    }
    fn plugin_disable_service(&self) -> Arc<dyn PluginDisableService> {
        self.disable_plugin_service()
    }
}

impl EventsFeature for AetherContainer {
    fn list_progress_bars_use_case(&self) -> Arc<dyn ListProgressBarsUseCasePort> {
        Arc::new(ListProgressBarsUseCase::new(
            self.progress_bar_storage.clone(),
        ))
    }
    fn event_emitter(&self) -> Arc<dyn EventEmitter<Event>> {
        self.event_emitter.clone()
    }
    fn progress_service(&self) -> Arc<dyn ProgressService> {
        self.progress_service.clone()
    }
}

impl FileWatcherFeature for AetherContainer {
    fn file_watcher(&self) -> Arc<dyn FileWatcher> {
        panic!("FileWatcher is not directly exposed; use InstanceWatcherService instead")
    }
    fn file_event_handler(&self) -> Arc<dyn FileEventHandler> {
        panic!("FileEventHandler is not directly exposed")
    }
}
