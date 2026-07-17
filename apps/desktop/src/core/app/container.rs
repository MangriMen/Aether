use std::collections::HashMap;
use std::sync::Arc;

use aether_core::core::app::{
    AetherContainer, AetherContainerParams, MinecraftParams, PluginParams, StorageParams,
};
use aether_core::features::auth::infra::SqliteCredentialsStorage;
use aether_core::features::events::ProgressServiceImpl;
use aether_core::features::events::SharedEventEmitter;
use aether_core::features::events::infra::InMemoryProgressBarStorage;
use aether_core::features::events::{EventEmitterExt, PluginEvent};
use aether_core::features::file_watcher::infra::NotifyFileWatcher;
use aether_core::features::instance::infra::{
    EventEmittingInstanceStorage, FsContentFileService, FsInstanceFileService,
    InstanceEventHandler, InstanceWatcherServiceImpl, ModrinthContentProvider,
    SqliteInstanceStorage, SqlitePackStorage,
};
use aether_core::features::instance::{
    ContentProvider, ContentSource, GetInstanceUseCase, Importer, InstanceCrudPort,
    PackLifecycleHandlerRegistry, Updater,
};
use aether_core::features::java::infra::{
    AzulJreProvider, FsJavaInstallationService, MemoryJavaInstallationTracker, SqliteJavaStorage,
};
use aether_core::features::minecraft::infra::{
    AssetsService, CachedMetadataStorage, ClientService, ForgeProcessor, LibrariesService,
    MinecraftDownloadResolver, MinecraftDownloadService, ModrinthMetadataStorage,
};
use aether_core::features::plugins::{
    ExtismPluginLoader, FsPluginSettingsStorage, FsPluginSourceStorage, FsPluginStorage,
    GithubProvider, PluginInfrastructureListener, ZipPluginExtractor,
};
use aether_core::features::plugins::{
    LoadConfigType, PluginLoader, PluginLoaderRegistry, PluginProvider, PluginProviderFactory,
    PluginRegistry,
};
use aether_core::features::process::infra::InMemoryProcessStorage;
use aether_core::features::settings::LocationInfo;
use aether_core::features::settings::Settings;
use aether_core::features::settings::SettingsStorage;
use aether_core::features::settings::infra::{
    FsDefaultInstanceSettingsStorage, FsSettingsStorage, SqliteDefaultInstanceSettingsStorage,
    SqliteSettingsStorage,
};
use aether_core::features::{auth, instance, java, minecraft, settings};
use aether_core::shared::cache::AssetsStorage;
use aether_core::shared::cache::infra::{FileCache, FsAssetsStorage, SqliteCache};
use aether_core::shared::capability::CapabilityRegistry;
use aether_core::shared::capability::infra::MemoryCapabilityRegistry;
use aether_core::shared::fetch::FetchSemaphore;
use aether_core::shared::request_client::infra::ReqwestClient;
use log::info;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use sqlx::SqlitePool;

type PBarStorage = InMemoryProgressBarStorage;
type ProgressServiceType = ProgressServiceImpl<PBarStorage>;

/// Build the complete `AetherContainer` with all concrete dependencies wired up.
///
/// This is the Composition Root for the desktop application. It creates every
/// infrastructure adapter (`SQLite`, filesystem, `HTTP`, etc.), performs data
/// migrations, and assembles the full dependency graph before returning an
/// `Arc<AetherContainer>` ready for use.
#[allow(clippy::too_many_lines)]
pub async fn build_container(
    location_info: Arc<LocationInfo>,
    event_emitter: SharedEventEmitter,
    pool: SqlitePool,
) -> anyhow::Result<Arc<AetherContainer>> {
    let config_dir = location_info.config_dir();
    let migrated_dir_name = "migrated";

    // ── FS → SQLite migrations ──────────────────────────────────────
    settings::infra::migrate_settings_to_sqlite(
        &FsSettingsStorage::new(config_dir),
        &SqliteSettingsStorage::new(pool.clone()),
        migrated_dir_name,
    )
    .await?;

    settings::infra::migrate_default_instance_settings_to_sqlite(
        &FsDefaultInstanceSettingsStorage::new(config_dir),
        &SqliteDefaultInstanceSettingsStorage::new(pool.clone()),
        migrated_dir_name,
    )
    .await?;

    auth::infra::migrate_credentials_to_sqlite(
        &aether_core::features::auth::infra::FsCredentialsStorage::new(config_dir),
        &SqliteCredentialsStorage::new(pool.clone()),
        migrated_dir_name,
    )
    .await?;

    instance::infra::migrate_instances_to_sqlite(
        &aether_core::features::instance::infra::FsInstanceStorage::new(location_info.clone()),
        &SqliteInstanceStorage::new(pool.clone()),
    )
    .await?;

    instance::infra::migrate_packs_to_sqlite(
        &aether_core::features::instance::infra::FsInstanceStorage::new(location_info.clone()),
        &aether_core::features::instance::infra::FsPackStorage::new(location_info.clone()),
        &SqlitePackStorage::new(pool.clone()),
    )
    .await?;

    java::infra::migrate_java_to_sqlite(
        &location_info.java_dir(),
        &FsJavaInstallationService,
        &SqliteJavaStorage::new(pool.clone()),
    )
    .await?;

    minecraft::infra::migrate_minecraft_metadata_to_sqlite(&location_info).await;

    // ── Settings bootstrap ──────────────────────────────────────────
    let settings_storage_sqlite = Arc::new(SqliteSettingsStorage::new(pool.clone()));
    let settings = if let Ok(s) = settings_storage_sqlite.get().await {
        s
    } else {
        let default = Settings::default();
        let _ = settings_storage_sqlite.upsert(default.clone()).await;
        default
    };
    let max_downloads = settings.max_concurrent_downloads();
    let fetch_semaphore = Arc::new(FetchSemaphore(tokio::sync::Semaphore::new(max_downloads)));

    // ── HTTP client ─────────────────────────────────────────────────
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

    // ── Progress & HTTP wrapper ─────────────────────────────────────
    let progress_bar_storage: Arc<PBarStorage> = Arc::new(InMemoryProgressBarStorage::default());
    let progress_service: Arc<ProgressServiceType> = Arc::new(ProgressServiceImpl::new(
        event_emitter.clone(),
        progress_bar_storage.clone(),
    ));
    let http_client: Arc<ReqwestClient<ProgressServiceType>> = Arc::new(ReqwestClient::new(
        progress_service.clone(),
        reqwest_client.clone(),
        fetch_semaphore,
    ));

    // ── Storage implementations ─────────────────────────────────────
    let credentials_storage: Arc<dyn aether_core::features::auth::CredentialsStorage> =
        Arc::new(SqliteCredentialsStorage::new(pool.clone()));

    let settings_storage: Arc<dyn aether_core::features::settings::SettingsStorage> =
        settings_storage_sqlite.clone();
    let default_instance_settings_storage: Arc<
        dyn aether_core::features::settings::DefaultInstanceSettingsStorage,
    > = Arc::new(SqliteDefaultInstanceSettingsStorage::new(pool.clone()));

    let process_storage: Arc<dyn aether_core::features::process::ProcessStorage> =
        Arc::new(InMemoryProcessStorage::default());

    let instance_storage: Arc<dyn aether_core::features::instance::InstanceStorage> =
        Arc::new(EventEmittingInstanceStorage::new(
            event_emitter.clone(),
            SqliteInstanceStorage::new(pool.clone()),
        ));
    let pack_storage: Arc<dyn aether_core::features::instance::PackStorage> =
        Arc::new(SqlitePackStorage::new(pool.clone()));
    let content_file_service: Arc<dyn aether_core::features::instance::ContentFileService> =
        Arc::new(FsContentFileService::new(location_info.clone()));
    let instance_file_service: Arc<dyn aether_core::features::instance::InstanceFileService> =
        Arc::new(FsInstanceFileService::new(location_info.clone()));

    let java_storage: Arc<dyn aether_core::features::java::JavaStorage> =
        Arc::new(SqliteJavaStorage::new(pool.clone()));
    let java_installation_service: Arc<dyn aether_core::features::java::JavaInstallationService> =
        Arc::new(FsJavaInstallationService);
    let java_installation_tracker: Arc<dyn aether_core::features::java::JavaInstallationTracker> =
        Arc::new(MemoryJavaInstallationTracker::default());
    let jre_provider: Arc<dyn aether_core::features::java::JreProvider> = Arc::new(
        AzulJreProvider::new(progress_service.clone(), http_client.clone()),
    );

    let metadata_storage: Arc<dyn aether_core::features::minecraft::MetadataStorage> =
        Arc::new(CachedMetadataStorage::new(
            SqliteCache::new(pool.clone()),
            ModrinthMetadataStorage::new(http_client.clone()),
        ));

    // ── Minecraft download chain ────────────────────────────────────
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
    let minecraft_downloader: Arc<dyn aether_core::features::minecraft::MinecraftDownloader> =
        Arc::new(MinecraftDownloadService::new(
            client_svc,
            assets_svc,
            libraries_svc,
            http_client.clone(),
            progress_service.clone(),
            minecraft_cache.clone(),
        ));
    let forge_processor: Arc<dyn aether_core::features::minecraft::ModLoaderProcessor> = Arc::new(
        ForgeProcessor::new(progress_service.clone(), location_info.clone()),
    );

    // ── Plugin infrastructure ───────────────────────────────────────
    let plugin_registry = Arc::new(PluginRegistry::new(event_emitter.clone()));
    let extism_loader = Arc::new(ExtismPluginLoader::new(location_info.clone()));
    let plugin_loader_registry = Arc::new(PluginLoaderRegistry::new(HashMap::from([(
        LoadConfigType::Extism,
        extism_loader.clone() as Arc<dyn PluginLoader>,
    )])));
    let plugin_storage: Arc<dyn aether_core::features::plugins::PluginStorage> =
        Arc::new(FsPluginStorage::new(location_info.clone(), None));
    let plugin_source_storage: Arc<dyn aether_core::features::plugins::PluginSourceStorage> =
        Arc::new(FsPluginSourceStorage::new(location_info.clone()));
    let plugin_settings_storage: Arc<dyn aether_core::features::plugins::PluginSettingsStorage> =
        Arc::new(FsPluginSettingsStorage::new(location_info.clone()));
    let plugin_provider_factory =
        Arc::new(PluginProviderFactory::new(vec![
            Box::new(GithubProvider::new(reqwest_client.clone())) as Box<dyn PluginProvider>,
        ]));
    let plugin_extractor: Arc<dyn aether_core::features::plugins::PluginExtractor> =
        Arc::new(ZipPluginExtractor::default());

    // ── Instance watcher ────────────────────────────────────────────
    let instance_watcher_service: Arc<dyn aether_core::features::instance::InstanceWatcherService> = {
        let event_handler = Arc::new(InstanceEventHandler::new(
            event_emitter.clone(),
            Arc::new(GetInstanceUseCase::new(instance_storage.clone())),
        ));
        let watcher = NotifyFileWatcher::new(event_handler)
            .map_err(|e| anyhow::anyhow!("File watcher: {e}"))?;
        Arc::new(InstanceWatcherServiceImpl::new(
            Arc::new(watcher),
            location_info.clone(),
        ))
    };

    // ── Capability registries ───────────────────────────────────────
    let importers_registry: Arc<dyn CapabilityRegistry<Arc<dyn Importer>>> =
        Arc::new(MemoryCapabilityRegistry::new("importer"));
    let updaters_registry: Arc<dyn CapabilityRegistry<Arc<dyn Updater>>> =
        Arc::new(MemoryCapabilityRegistry::new("updater"));
    let content_provider_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentProvider>>> =
        Arc::new(MemoryCapabilityRegistry::new("content_provider"));
    let content_source_registry: Arc<dyn CapabilityRegistry<Arc<dyn ContentSource>>> =
        Arc::new(MemoryCapabilityRegistry::new("content_source"));
    let pack_lifecycle_handler_registry = Arc::new(PackLifecycleHandlerRegistry::new());

    // ── Assets storage ──────────────────────────────────────────────
    let file_cache_assets = Arc::new(FileCache::new(
        aether_core::shared::cache::infra::AssetsResolver::new(location_info.clone()),
    ));
    let assets_storage: Arc<dyn AssetsStorage> = Arc::new(FsAssetsStorage::new(file_cache_assets));

    // ── Assemble container ──────────────────────────────────────────
    let container = AetherContainer::new(AetherContainerParams {
        storage: StorageParams {
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
            assets_storage,
        },
        minecraft: MinecraftParams {
            minecraft_downloader,
            mod_loader_processor: forge_processor,
        },
        plugins: PluginParams {
            registry: plugin_registry.clone(),
            loader_registry: plugin_loader_registry,
            storage: plugin_storage,
            source_storage: plugin_source_storage,
            settings_storage: plugin_settings_storage,
            provider_factory: plugin_provider_factory,
            extractor: plugin_extractor,
        },
        event_emitter: event_emitter.clone(),
        progress_bar_storage,
        progress_service: progress_service
            as Arc<dyn aether_core::features::events::ProgressService>,
        location_info: location_info.clone(),
        instance_watcher_service: instance_watcher_service.clone(),
        importers_registry: importers_registry.clone(),
        updaters_registry: updaters_registry.clone(),
        content_provider_registry: content_provider_registry.clone(),
        content_source_registry: content_source_registry.clone(),
        pack_lifecycle_handler_registry: pack_lifecycle_handler_registry.clone(),
        request_client: http_client.clone(),
    });

    // ── Post-construction wiring ────────────────────────────────────
    extism_loader.set_container(&container);

    instance_watcher_service
        .watch_instances()
        .await
        .map_err(|e| anyhow::anyhow!("Instance watcher: {e}"))?;

    // Register ModrinthContentProvider (as ContentProvider + ContentSource)
    {
        let provider = Arc::new(ModrinthContentProvider::new(
            location_info.clone(),
            None,
            http_client.clone(),
            container.create_instance_use_case(),
        ));
        let meta = ContentSource::metadata(&*provider);
        let _ = content_provider_registry
            .add(
                ModrinthContentProvider::ID.to_owned(),
                meta.id.clone(),
                provider.clone(),
            )
            .await;
        // Also register as ContentSource
        let _ = content_source_registry
            .add(
                ModrinthContentProvider::ID.to_owned(),
                "modrinth-content".to_owned(),
                provider,
            )
            .await;
    }

    // Set up plugin infrastructure listener
    let plugin_infra_listener = Arc::new(PluginInfrastructureListener::new(
        plugin_registry,
        importers_registry,
        updaters_registry,
        content_provider_registry,
        content_source_registry,
    ));
    event_emitter.on::<PluginEvent, _>({
        let listener = plugin_infra_listener.clone();
        move |event| {
            let task = listener.clone();
            tokio::spawn(async move { task.on_plugin_event(event).await });
        }
    });

    info!("AetherContainer initialized via Composition Root");
    Ok(container)
}
