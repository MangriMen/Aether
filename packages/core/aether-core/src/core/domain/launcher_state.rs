use std::sync::Arc;

use tokio::sync::{OnceCell, Semaphore};

use crate::{
    core::domain::LazyLocator,
    features::{
        auth::{
            self,
            infra::{FsCredentialsStorage, SqliteCredentialsStorage},
        },
        events::{EventEmitterExt, PluginEvent, SharedEventEmitter},
        instance::{
            self, InstanceWatcherService,
            infra::{FsInstanceStorage, FsPackStorage, SqliteInstanceStorage, SqlitePackStorage},
        },
        java::{
            self,
            infra::{FsJavaInstallationService, SqliteJavaStorage},
        },
        minecraft,
        settings::{
            self, LocationInfo, Settings, SettingsStorage,
            infra::{
                FsDefaultInstanceSettingsStorage, FsSettingsStorage,
                SqliteDefaultInstanceSettingsStorage, SqliteSettingsStorage,
            },
        },
    },
    shared::domain::FetchSemaphore,
};

// Global state
// RwLock on state only has concurrent reads, except for config dir change which takes control of the State
static LAUNCHER_STATE: OnceCell<Arc<LauncherState>> = OnceCell::const_new();

#[derive(Debug)]
pub struct LauncherState {
    // Information about files location
    pub location_info: Arc<LocationInfo>,

    /// Semaphore used to limit concurrent network requests and avoid errors
    pub fetch_semaphore: Arc<FetchSemaphore>,

    // /// Semaphore used to limit concurrent I/O and avoid errors
    // pub io_semaphore: IoSemaphore,

    // ///
    /// Semaphore to limit concurrent API requests. This is separate from the fetch semaphore
    /// to keep API functionality while the app is performing intensive tasks.
    pub api_semaphore: Arc<FetchSemaphore>,
}

impl LauncherState {
    pub async fn init(
        location_info: Arc<LocationInfo>,
        event_emitter: SharedEventEmitter,
        sqlite_pool: sqlx::SqlitePool,
    ) -> crate::Result<()> {
        LAUNCHER_STATE
            .get_or_try_init(|| Self::initialize(location_info, event_emitter, sqlite_pool))
            .await?;

        Ok(())
    }

    pub async fn get() -> crate::Result<Arc<Self>> {
        if !LAUNCHER_STATE.initialized() {
            tracing::error!(
                "Attempted to get state before it is initialized - this should never happen!\n{:?}",
                std::backtrace::Backtrace::capture()
            );

            while !LAUNCHER_STATE.initialized() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }

        Ok(Arc::clone(
            LAUNCHER_STATE.get().expect("State is not initialized!"),
        ))
    }

    pub async fn initialized() -> bool {
        LAUNCHER_STATE.initialized()
    }

    #[tracing::instrument(skip(event_emitter, sqlite_pool))]
    async fn initialize(
        location_info: Arc<LocationInfo>,
        event_emitter: SharedEventEmitter,
        sqlite_pool: sqlx::SqlitePool,
    ) -> crate::Result<Arc<Self>> {
        let settings_storage = SqliteSettingsStorage::new(sqlite_pool.clone());

        let migrated_dir_name = "migrated";

        let config_dir = location_info.config_dir();

        settings::infra::migrate_settings_to_sqlite(
            &FsSettingsStorage::new(config_dir),
            &settings_storage,
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

        let settings = if let Ok(settings) = settings_storage.get().await {
            settings
        } else {
            settings_storage.upsert(Settings::default()).await?
        };

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

        let fetch_semaphore = Arc::new(FetchSemaphore(Semaphore::new(
            settings.max_concurrent_downloads(),
        )));

        let api_semaphore = Arc::new(FetchSemaphore(Semaphore::new(
            settings.max_concurrent_downloads(),
        )));

        log::info!("State initialized");

        let state = Arc::new(Self {
            location_info,
            fetch_semaphore,
            api_semaphore,
        });

        LazyLocator::init(state.clone(), event_emitter, sqlite_pool).await?;

        let lazy_locator = LazyLocator::get().await?;
        lazy_locator
            .get_instance_watcher_service()
            .await?
            .watch_instances()
            .await?;

        let plugin_infra_listener = lazy_locator.get_plugin_infrastructure_listener().await;
        lazy_locator
            .get_event_emitter()
            .await
            .on::<PluginEvent, _>({
                let listener = plugin_infra_listener.clone();

                move |event| {
                    let listener_task = listener.clone();
                    tokio::spawn(async move { listener_task.on_plugin_event(event).await });
                }
            });

        log::info!("Service locator initialized");

        Ok(state)
    }
}
