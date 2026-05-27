use std::{collections::HashMap, sync::Arc};

use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use tokio::sync::OnceCell;

use crate::{
    features::{
        auth::{
            self,
            infra::{FsCredentialsStorage, SqliteCredentialsStorage},
        },
        events::{
            EventEmitterExt, PluginEvent, ProgressServiceImpl, SharedEventEmitter,
            infra::InMemoryProgressBarStorage,
        },
        file_watcher::infra::NotifyFileWatcher,
        instance::{
            self, ContentProvider, Importer, InstanceWatcherService, Updater,
            infra::{
                EventEmittingInstanceStorage, FsInstanceStorage, FsPackStorage,
                InstanceEventHandler, InstanceWatcherServiceImpl, ModrinthContentProvider,
                SqliteInstanceStorage, SqlitePackStorage,
            },
        },
        java::{
            self,
            infra::{FsJavaInstallationService, MemoryJavaInstallationTracker, SqliteJavaStorage},
        },
        minecraft::{
            self,
            infra::{CachedMetadataStorage, ModrinthMetadataStorage},
        },
        plugins::{
            LoadConfigType, PluginLoaderRegistry, PluginRegistry,
            infra::{
                ExtismPluginLoader, FsPluginSettingsStorage, FsPluginStorage,
                PluginInfrastructureListener, ZipPluginExtractor,
            },
        },
        process::infra::InMemoryProcessStorage,
        settings::{
            self, LocationInfo, Settings, SettingsStorage,
            infra::{
                FsDefaultInstanceSettingsStorage, FsSettingsStorage,
                SqliteDefaultInstanceSettingsStorage, SqliteSettingsStorage,
            },
        },
    },
    libs::request_client::ReqwestClient,
    shared::{CapabilityRegistry, FetchSemaphore, MemoryCapabilityRegistry, SqliteCache},
};

use super::ErrorKind;

static LAZY_LOCATOR: OnceCell<Arc<LazyLocator>> = OnceCell::const_new();

pub type ProgressServiceType = ProgressServiceImpl<InMemoryProgressBarStorage>;

pub type ImporterRegistry = MemoryCapabilityRegistry<Arc<dyn Importer>>;
pub type UpdaterRegistry = MemoryCapabilityRegistry<Arc<dyn Updater>>;
pub type ContentProviderRegistry = MemoryCapabilityRegistry<Arc<dyn ContentProvider>>;

pub type MinecraftMetadataCache = SqliteCache;

pub struct LazyLocator {
    pub location_info: Arc<LocationInfo>,

    /// Semaphore used to limit concurrent network requests and avoid errors
    pub fetch_semaphore: Arc<FetchSemaphore>,

    // /// Semaphore used to limit concurrent I/O and avoid errors
    // pub io_semaphore: IoSemaphore,

    // ///
    /// Semaphore to limit concurrent API requests. This is separate from the fetch semaphore
    /// to keep API functionality while the app is performing intensive tasks.
    pub api_semaphore: Arc<FetchSemaphore>,

    reqwest_client: Arc<ClientWithMiddleware>,
    request_client: OnceCell<Arc<ReqwestClient<ProgressServiceType>>>,
    api_client: OnceCell<Arc<ReqwestClient<ProgressServiceType>>>,
    credentials_storage: OnceCell<Arc<SqliteCredentialsStorage>>,
    settings_storage: OnceCell<Arc<SqliteSettingsStorage>>,
    process_storage: OnceCell<Arc<InMemoryProcessStorage>>,
    instance_storage: OnceCell<Arc<EventEmittingInstanceStorage<SqliteInstanceStorage>>>,
    java_storage: OnceCell<Arc<SqliteJavaStorage>>,
    metadata_storage: OnceCell<
        Arc<
            CachedMetadataStorage<
                MinecraftMetadataCache,
                ModrinthMetadataStorage<ReqwestClient<ProgressServiceType>>,
            >,
        >,
    >,
    pack_storage: OnceCell<Arc<SqlitePackStorage>>,
    plugin_settings_storage: OnceCell<Arc<FsPluginSettingsStorage>>,
    plugin_registry: OnceCell<Arc<PluginRegistry>>,
    plugin_loader_registry: OnceCell<Arc<PluginLoaderRegistry<ExtismPluginLoader>>>,
    plugin_storage: OnceCell<Arc<FsPluginStorage>>,
    event_emitter: OnceCell<SharedEventEmitter>,
    progress_bar_storage: OnceCell<Arc<InMemoryProgressBarStorage>>,
    progress_service: OnceCell<Arc<ProgressServiceType>>,
    instance_watcher_service:
        OnceCell<Arc<InstanceWatcherServiceImpl<NotifyFileWatcher<InstanceEventHandler>>>>,
    default_instance_settings_storage: OnceCell<Arc<SqliteDefaultInstanceSettingsStorage>>,
    plugin_extractor: OnceCell<Arc<ZipPluginExtractor>>,
    importers_registry: OnceCell<Arc<ImporterRegistry>>,
    updaters_registry: OnceCell<Arc<UpdaterRegistry>>,
    content_provider_registry: OnceCell<Arc<ContentProviderRegistry>>,
    plugin_infrastructure_listener: OnceCell<
        Arc<
            PluginInfrastructureListener<
                ImporterRegistry,
                UpdaterRegistry,
                ContentProviderRegistry,
            >,
        >,
    >,
    sqlite_pool: OnceCell<sqlx::SqlitePool>,
    java_installation_tracker: OnceCell<Arc<MemoryJavaInstallationTracker>>,
}

fn get_reqwest_client() -> Arc<ClientWithMiddleware> {
    const FETCH_ATTEMPTS: u32 = 5;
    const TCP_KEEP_ALIVE_TIME: std::time::Duration = std::time::Duration::from_secs(10);

    let client = reqwest::Client::builder()
        .tcp_keepalive(Some(TCP_KEEP_ALIVE_TIME))
        .build()
        .expect("Failed to build reqwest client");

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(FETCH_ATTEMPTS);
    let retry_middleware = reqwest_retry::RetryTransientMiddleware::new_with_policy(retry_policy);

    let client_with_middlewares = reqwest_middleware::ClientBuilder::new(client)
        .with(retry_middleware)
        .build();

    Arc::new(client_with_middlewares)
}

impl LazyLocator {
    pub async fn init(
        location_info: Arc<LocationInfo>,
        event_emitter: SharedEventEmitter,
        sqlite_pool: sqlx::SqlitePool,
    ) -> crate::Result<()> {
        Self::run_migrations(location_info.clone(), &sqlite_pool).await?;

        let settings_storage = SqliteSettingsStorage::new(sqlite_pool.clone());
        let (fetch_semaphore, api_semaphore) = Self::create_semaphores(&settings_storage).await?;

        LAZY_LOCATOR
            .get_or_init(|| async {
                Arc::new(Self {
                    location_info,
                    fetch_semaphore,
                    api_semaphore,
                    reqwest_client: get_reqwest_client(),
                    request_client: OnceCell::new(),
                    api_client: OnceCell::new(),
                    credentials_storage: OnceCell::new(),
                    settings_storage: OnceCell::new(),
                    process_storage: OnceCell::new(),
                    instance_storage: OnceCell::new(),
                    java_storage: OnceCell::new(),
                    metadata_storage: OnceCell::new(),
                    pack_storage: OnceCell::new(),
                    plugin_settings_storage: OnceCell::new(),
                    plugin_registry: OnceCell::new(),
                    plugin_loader_registry: OnceCell::new(),
                    plugin_storage: OnceCell::new(),
                    event_emitter: OnceCell::from(event_emitter),
                    progress_bar_storage: OnceCell::new(),
                    progress_service: OnceCell::new(),
                    instance_watcher_service: OnceCell::new(),
                    default_instance_settings_storage: OnceCell::new(),
                    plugin_extractor: OnceCell::new(),
                    importers_registry: OnceCell::new(),
                    updaters_registry: OnceCell::new(),
                    content_provider_registry: OnceCell::new(),
                    plugin_infrastructure_listener: OnceCell::new(),
                    sqlite_pool: OnceCell::from(sqlite_pool),
                    java_installation_tracker: OnceCell::new(),
                })
            })
            .await;

        Self::start_infrastructure_services().await?;

        log::info!("LazyLocator locator and infrastructure services initialized");

        Ok(())
    }

    pub async fn get() -> crate::Result<Arc<Self>> {
        if !LAZY_LOCATOR.initialized() {
            tracing::error!(
                "Attempted to get LazyLocator before it is initialized - this should never happen!",
            );
            tracing::error!("{}", std::backtrace::Backtrace::capture());

            Self::wait_for_initialization().await?;
        }

        LAZY_LOCATOR.get().map(Arc::clone).ok_or_else(|| {
            ErrorKind::CoreError("LazyLocator is not initialized!".to_string()).as_error()
        })
    }

    pub async fn initialized() -> bool {
        LAZY_LOCATOR.initialized()
    }

    async fn wait_for_initialization() -> crate::Result<()> {
        const INIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
        const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

        let start = std::time::Instant::now();

        while !LAZY_LOCATOR.initialized() {
            if start.elapsed() > INIT_TIMEOUT {
                return Err(
                    ErrorKind::CoreError("LazyLocator initialization timeout".to_string())
                        .as_error(),
                );
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }

        Ok(())
    }

    async fn run_migrations(
        location_info: Arc<LocationInfo>,
        sqlite_pool: &sqlx::SqlitePool,
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
            &FsInstanceStorage::new(location_info.clone()),
            &SqliteInstanceStorage::new(sqlite_pool.clone()),
        )
        .await?;

        instance::infra::migrate_packs_to_sqlite(
            &FsInstanceStorage::new(location_info.clone()),
            &FsPackStorage::new(location_info.clone()),
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

        Ok(())
    }

    async fn create_semaphores(
        settings_storage: &SqliteSettingsStorage,
    ) -> crate::Result<(Arc<FetchSemaphore>, Arc<FetchSemaphore>)> {
        let settings = if let Ok(settings) = settings_storage.get().await {
            settings
        } else {
            settings_storage.upsert(Settings::default()).await?
        };

        let max_downloads = settings.max_concurrent_downloads();

        let fetch_semaphore = Arc::new(FetchSemaphore(tokio::sync::Semaphore::new(max_downloads)));
        let api_semaphore = Arc::new(FetchSemaphore(tokio::sync::Semaphore::new(max_downloads)));

        Ok((fetch_semaphore, api_semaphore))
    }

    async fn start_infrastructure_services() -> crate::Result<()> {
        let locator = Self::get().await?;

        locator
            .get_instance_watcher_service()
            .await?
            .watch_instances()
            .await?;

        let plugin_infra_listener = locator.get_plugin_infrastructure_listener().await;
        locator.get_event_emitter().await.on::<PluginEvent, _>({
            let listener = plugin_infra_listener.clone();
            move |event| {
                let listener_task = listener.clone();
                tokio::spawn(async move { listener_task.on_plugin_event(event).await });
            }
        });

        Ok(())
    }

    pub async fn get_request_client(&self) -> Arc<ReqwestClient<ProgressServiceType>> {
        self.request_client
            .get_or_init(|| async {
                Arc::new(ReqwestClient::new(
                    self.get_progress_service().await,
                    self.reqwest_client.clone(),
                    self.fetch_semaphore.clone(),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_api_client(&self) -> Arc<ReqwestClient<ProgressServiceType>> {
        self.api_client
            .get_or_init(|| async {
                Arc::new(ReqwestClient::new(
                    self.get_progress_service().await,
                    self.reqwest_client.clone(),
                    self.api_semaphore.clone(),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_credentials_storage(&self) -> Arc<SqliteCredentialsStorage> {
        self.credentials_storage
            .get_or_init(|| async {
                Arc::new(SqliteCredentialsStorage::new(self.get_sqlite_pool().await))
            })
            .await
            .clone()
    }

    pub async fn get_settings_storage(&self) -> Arc<SqliteSettingsStorage> {
        self.settings_storage
            .get_or_init(|| async {
                Arc::new(SqliteSettingsStorage::new(self.get_sqlite_pool().await))
            })
            .await
            .clone()
    }

    pub async fn get_process_storage(&self) -> Arc<InMemoryProcessStorage> {
        self.process_storage
            .get_or_init(|| async { Arc::new(InMemoryProcessStorage::default()) })
            .await
            .clone()
    }

    pub async fn get_instance_storage(
        &self,
    ) -> Arc<EventEmittingInstanceStorage<SqliteInstanceStorage>> {
        self.instance_storage
            .get_or_init(|| async {
                Arc::new(EventEmittingInstanceStorage::new(
                    self.get_event_emitter().await,
                    SqliteInstanceStorage::new(self.get_sqlite_pool().await),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_java_storage(&self) -> Arc<SqliteJavaStorage> {
        self.java_storage
            .get_or_init(|| async {
                Arc::new(SqliteJavaStorage::new(self.get_sqlite_pool().await))
            })
            .await
            .clone()
    }

    pub async fn get_metadata_storage(
        &self,
    ) -> Arc<
        CachedMetadataStorage<
            MinecraftMetadataCache,
            ModrinthMetadataStorage<ReqwestClient<ProgressServiceType>>,
        >,
    > {
        self.metadata_storage
            .get_or_init(|| async {
                Arc::new(CachedMetadataStorage::new(
                    SqliteCache::new(self.get_sqlite_pool().await),
                    ModrinthMetadataStorage::new(self.get_request_client().await),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_pack_storage(&self) -> Arc<SqlitePackStorage> {
        self.pack_storage
            .get_or_init(|| async {
                Arc::new(SqlitePackStorage::new(self.get_sqlite_pool().await))
            })
            .await
            .clone()
    }

    pub async fn get_plugin_settings_storage(&self) -> Arc<FsPluginSettingsStorage> {
        self.plugin_settings_storage
            .get_or_init(|| async {
                Arc::new(FsPluginSettingsStorage::new(self.location_info.clone()))
            })
            .await
            .clone()
    }

    pub async fn get_plugin_registry(&self) -> Arc<PluginRegistry> {
        self.plugin_registry
            .get_or_init(|| async { Arc::new(PluginRegistry::new(self.get_event_emitter().await)) })
            .await
            .clone()
    }

    pub async fn get_plugin_loader_registry(
        &self,
    ) -> Arc<PluginLoaderRegistry<ExtismPluginLoader>> {
        self.plugin_loader_registry
            .get_or_init(|| async {
                let loaders = HashMap::from([(
                    LoadConfigType::Extism,
                    ExtismPluginLoader::new(self.location_info.clone()),
                )]);

                Arc::new(PluginLoaderRegistry::new(loaders))
            })
            .await
            .clone()
    }

    pub async fn get_plugin_storage(&self) -> Arc<FsPluginStorage> {
        self.plugin_storage
            .get_or_init(|| async {
                Arc::new(FsPluginStorage::new(self.location_info.clone(), None))
            })
            .await
            .clone()
    }

    pub async fn get_event_emitter(&self) -> SharedEventEmitter {
        self.event_emitter
            .get()
            .expect("EventEmitter must be initialized via LazyLocator::init")
            .clone()
    }

    pub async fn get_progress_bar_storage(&self) -> Arc<InMemoryProgressBarStorage> {
        self.progress_bar_storage
            .get_or_init(|| async { Arc::new(InMemoryProgressBarStorage::default()) })
            .await
            .clone()
    }

    pub async fn get_progress_service(&self) -> Arc<ProgressServiceType> {
        self.progress_service
            .get_or_init(|| async {
                Arc::new(ProgressServiceImpl::new(
                    self.get_event_emitter().await,
                    self.get_progress_bar_storage().await,
                ))
            })
            .await
            .clone()
    }

    pub async fn get_instance_watcher_service(
        &self,
    ) -> crate::Result<Arc<InstanceWatcherServiceImpl<NotifyFileWatcher<InstanceEventHandler>>>>
    {
        self.instance_watcher_service
            .get_or_try_init(|| async {
                let watcher = NotifyFileWatcher::new(Arc::new(InstanceEventHandler::new(
                    self.get_event_emitter().await,
                )))?;

                let service =
                    InstanceWatcherServiceImpl::new(Arc::new(watcher), self.location_info.clone());

                Ok(Arc::new(service))
            })
            .await
            .cloned()
    }

    pub async fn get_default_instance_settings_storage(
        &self,
    ) -> Arc<SqliteDefaultInstanceSettingsStorage> {
        self.default_instance_settings_storage
            .get_or_init(|| async {
                Arc::new(SqliteDefaultInstanceSettingsStorage::new(
                    self.get_sqlite_pool().await,
                ))
            })
            .await
            .clone()
    }

    pub async fn get_plugin_extractor(&self) -> Arc<ZipPluginExtractor> {
        self.plugin_extractor
            .get_or_init(|| async { Arc::new(ZipPluginExtractor::default()) })
            .await
            .clone()
    }

    pub async fn get_importers_registry(&self) -> Arc<ImporterRegistry> {
        self.importers_registry
            .get_or_init(|| async { Arc::new(MemoryCapabilityRegistry::new("importer")) })
            .await
            .clone()
    }

    pub async fn get_updaters_registry(&self) -> Arc<UpdaterRegistry> {
        self.updaters_registry
            .get_or_init(|| async { Arc::new(MemoryCapabilityRegistry::new("updater")) })
            .await
            .clone()
    }

    pub async fn get_content_provider_registry(&self) -> Arc<ContentProviderRegistry> {
        self.content_provider_registry
            .get_or_init(|| async {
                let registry: Arc<ContentProviderRegistry> =
                    Arc::new(MemoryCapabilityRegistry::new("content_provider"));

                let providers = [Arc::new(ModrinthContentProvider::new(
                    self.location_info.clone(),
                    None,
                    self.get_request_client().await,
                ))];

                for provider in providers {
                    let meta = provider.metadata();

                    if let Err(err) = registry
                        .add(
                            ModrinthContentProvider::ID.to_owned(),
                            meta.id.clone(),
                            provider.clone(),
                        )
                        .await
                    {
                        tracing::error!(
                            "Failed to register content provider {}: {}",
                            meta.id.clone(),
                            err
                        );
                    }
                }

                registry
            })
            .await
            .clone()
    }

    pub async fn get_plugin_infrastructure_listener(
        &self,
    ) -> Arc<PluginInfrastructureListener<ImporterRegistry, UpdaterRegistry, ContentProviderRegistry>>
    {
        self.plugin_infrastructure_listener
            .get_or_init(|| async {
                Arc::new(PluginInfrastructureListener::new(
                    self.get_plugin_registry().await,
                    self.get_importers_registry().await,
                    self.get_updaters_registry().await,
                    self.get_content_provider_registry().await,
                ))
            })
            .await
            .clone()
    }

    pub async fn get_sqlite_pool(&self) -> sqlx::SqlitePool {
        self.sqlite_pool
            .get()
            .expect("Sqlite pool must be initialized via LazyLocator::init")
            .clone()
    }

    pub async fn get_java_installation_tracker(&self) -> Arc<MemoryJavaInstallationTracker> {
        self.java_installation_tracker
            .get_or_init(|| async { Arc::new(MemoryJavaInstallationTracker::default()) })
            .await
            .clone()
    }
}
