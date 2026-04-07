use std::{collections::HashMap, sync::Arc};

use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use tokio::sync::OnceCell;

use crate::{
    features::{
        auth::infra::FsCredentialsStorage,
        events::{
            infra::{InMemoryProgressBarStorage, TauriEventEmitter},
            ProgressServiceImpl,
        },
        file_watcher::infra::NotifyFileWatcher,
        instance::{
            infra::{
                EventEmittingInstanceStorage, FsInstanceStorage, FsPackStorage,
                InstanceEventHandler, ModrinthContentProvider,
            },
            ContentProvider, Importer, InstanceWatcherServiceImpl, Updater,
        },
        java::infra::FsJavaStorage,
        minecraft::infra::{
            CachedMetadataStorage, MinecraftMetadataResolver, ModrinthMetadataStorage,
        },
        plugins::{
            infra::{
                ExtismPluginLoader, FsPluginSettingsStorage, FsPluginStorage,
                PluginInfrastructureListener, ZipPluginExtractor,
            },
            LoadConfigType, PluginLoaderRegistry, PluginRegistry,
        },
        process::infra::InMemoryProcessStorage,
        settings::infra::{FsDefaultInstanceSettingsStorage, FsSettingsStorage},
    },
    libs::request_client::ReqwestClient,
    shared::{CapabilityRegistry, FileCache, MemoryCapabilityRegistry},
};

use super::{ErrorKind, LauncherState};

static LAZY_LOCATOR: OnceCell<Arc<LazyLocator>> = OnceCell::const_new();

pub type ProgressServiceType = ProgressServiceImpl<TauriEventEmitter, InMemoryProgressBarStorage>;

pub type ImporterRegistry = MemoryCapabilityRegistry<Arc<dyn Importer>>;
pub type UpdaterRegistry = MemoryCapabilityRegistry<Arc<dyn Updater>>;
pub type ContentProviderRegistry = MemoryCapabilityRegistry<Arc<dyn ContentProvider>>;

pub type MinecraftMetadataCache = FileCache<MinecraftMetadataResolver>;

pub struct LazyLocator {
    state: Arc<LauncherState>,
    app_handle: tauri::AppHandle,
    reqwest_client: Arc<ClientWithMiddleware>,
    request_client: OnceCell<Arc<ReqwestClient<ProgressServiceType>>>,
    api_client: OnceCell<Arc<ReqwestClient<ProgressServiceType>>>,
    credentials_storage: OnceCell<Arc<FsCredentialsStorage>>,
    settings_storage: OnceCell<Arc<FsSettingsStorage>>,
    process_storage: OnceCell<Arc<InMemoryProcessStorage>>,
    instance_storage:
        OnceCell<Arc<EventEmittingInstanceStorage<TauriEventEmitter, FsInstanceStorage>>>,
    java_storage: OnceCell<Arc<FsJavaStorage>>,
    metadata_storage: OnceCell<
        Arc<
            CachedMetadataStorage<
                MinecraftMetadataCache,
                ModrinthMetadataStorage<ReqwestClient<ProgressServiceType>>,
            >,
        >,
    >,
    pack_storage: OnceCell<Arc<FsPackStorage>>,
    plugin_settings_storage: OnceCell<Arc<FsPluginSettingsStorage>>,
    plugin_registry: OnceCell<Arc<PluginRegistry<TauriEventEmitter>>>,
    plugin_loader_registry: OnceCell<Arc<PluginLoaderRegistry<ExtismPluginLoader>>>,
    plugin_storage: OnceCell<Arc<FsPluginStorage>>,
    event_emitter: OnceCell<Arc<TauriEventEmitter>>,
    progress_bar_storage: OnceCell<Arc<InMemoryProgressBarStorage>>,
    progress_service: OnceCell<Arc<ProgressServiceType>>,
    instance_watcher_service: OnceCell<
        Arc<InstanceWatcherServiceImpl<NotifyFileWatcher<InstanceEventHandler<TauriEventEmitter>>>>,
    >,
    default_instance_settings_storage: OnceCell<Arc<FsDefaultInstanceSettingsStorage>>,
    plugin_extractor: OnceCell<Arc<ZipPluginExtractor>>,
    importers_registry: OnceCell<Arc<ImporterRegistry>>,
    updaters_registry: OnceCell<Arc<UpdaterRegistry>>,
    content_provider_registry: OnceCell<Arc<ContentProviderRegistry>>,
    plugin_infrastructure_listener: OnceCell<
        Arc<
            PluginInfrastructureListener<
                TauriEventEmitter,
                ImporterRegistry,
                UpdaterRegistry,
                ContentProviderRegistry,
            >,
        >,
    >,
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
        state: Arc<LauncherState>,
        app_handle: tauri::AppHandle,
    ) -> crate::Result<()> {
        LAZY_LOCATOR
            .get_or_init(|| async {
                Arc::new(Self {
                    state,
                    app_handle,
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
                    event_emitter: OnceCell::new(),
                    progress_bar_storage: OnceCell::new(),
                    progress_service: OnceCell::new(),
                    instance_watcher_service: OnceCell::new(),
                    default_instance_settings_storage: OnceCell::new(),
                    plugin_extractor: OnceCell::new(),
                    importers_registry: OnceCell::new(),
                    updaters_registry: OnceCell::new(),
                    content_provider_registry: OnceCell::new(),
                    plugin_infrastructure_listener: OnceCell::new(),
                })
            })
            .await;

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

    pub async fn get_request_client(&self) -> Arc<ReqwestClient<ProgressServiceType>> {
        self.request_client
            .get_or_init(|| async {
                Arc::new(ReqwestClient::new(
                    self.get_progress_service().await,
                    self.reqwest_client.clone(),
                    self.state.fetch_semaphore.clone(),
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
                    self.state.api_semaphore.clone(),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_credentials_storage(&self) -> Arc<FsCredentialsStorage> {
        self.credentials_storage
            .get_or_init(|| async {
                Arc::new(FsCredentialsStorage::new(
                    self.state.location_info.settings_dir(),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_settings_storage(&self) -> Arc<FsSettingsStorage> {
        self.settings_storage
            .get_or_init(|| async {
                Arc::new(FsSettingsStorage::new(
                    self.state.location_info.settings_dir(),
                ))
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
    ) -> Arc<EventEmittingInstanceStorage<TauriEventEmitter, FsInstanceStorage>> {
        self.instance_storage
            .get_or_init(|| async {
                Arc::new(EventEmittingInstanceStorage::new(
                    self.get_event_emitter().await,
                    FsInstanceStorage::new(self.state.location_info.clone()),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_java_storage(&self) -> Arc<FsJavaStorage> {
        self.java_storage
            .get_or_init(|| async {
                Arc::new(FsJavaStorage::new(&self.state.location_info.java_dir()))
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
                    FileCache::new(MinecraftMetadataResolver::new(
                        self.state.location_info.clone(),
                    )),
                    ModrinthMetadataStorage::new(self.get_request_client().await),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_pack_storage(&self) -> Arc<FsPackStorage> {
        self.pack_storage
            .get_or_init(|| async {
                Arc::new(FsPackStorage::new(self.state.location_info.clone()))
            })
            .await
            .clone()
    }

    pub async fn get_plugin_settings_storage(&self) -> Arc<FsPluginSettingsStorage> {
        self.plugin_settings_storage
            .get_or_init(|| async {
                Arc::new(FsPluginSettingsStorage::new(
                    self.state.location_info.clone(),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_plugin_registry(&self) -> Arc<PluginRegistry<TauriEventEmitter>> {
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
                    ExtismPluginLoader::new(self.state.location_info.clone()),
                )]);

                Arc::new(PluginLoaderRegistry::new(loaders))
            })
            .await
            .clone()
    }

    pub async fn get_plugin_storage(&self) -> Arc<FsPluginStorage> {
        self.plugin_storage
            .get_or_init(|| async {
                Arc::new(FsPluginStorage::new(self.state.location_info.clone(), None))
            })
            .await
            .clone()
    }

    pub async fn get_event_emitter(&self) -> Arc<TauriEventEmitter> {
        self.event_emitter
            .get_or_init(|| async { Arc::new(TauriEventEmitter::new(self.app_handle.clone())) })
            .await
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
    ) -> crate::Result<
        Arc<InstanceWatcherServiceImpl<NotifyFileWatcher<InstanceEventHandler<TauriEventEmitter>>>>,
    > {
        self.instance_watcher_service
            .get_or_try_init(|| async {
                let watcher = NotifyFileWatcher::new(Arc::new(InstanceEventHandler::new(
                    self.get_event_emitter().await,
                )))?;

                let service = InstanceWatcherServiceImpl::new(
                    Arc::new(watcher),
                    self.state.location_info.clone(),
                );

                Ok(Arc::new(service))
            })
            .await
            .cloned()
    }

    pub async fn get_default_instance_settings_storage(
        &self,
    ) -> Arc<FsDefaultInstanceSettingsStorage> {
        self.default_instance_settings_storage
            .get_or_init(|| async {
                Arc::new(FsDefaultInstanceSettingsStorage::new(
                    self.state.location_info.settings_dir(),
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
                    self.state.location_info.clone(),
                    None,
                    self.get_request_client().await,
                ))];

                for provider in providers {
                    let meta = provider.metadata();

                    if let Err(err) = registry
                        .add(
                            ModrinthContentProvider::ID.to_owned(),
                            meta.id.to_string(),
                            provider.clone(),
                        )
                        .await
                    {
                        tracing::error!(
                            "Failed to register content provider {}: {}",
                            meta.id.to_string(),
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
    ) -> Arc<
        PluginInfrastructureListener<
            TauriEventEmitter,
            ImporterRegistry,
            UpdaterRegistry,
            ContentProviderRegistry,
        >,
    > {
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
}
