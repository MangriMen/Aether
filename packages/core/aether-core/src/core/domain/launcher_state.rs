use std::{path::PathBuf, sync::Arc};

use tokio::sync::{OnceCell, Semaphore};

use crate::{
    core::domain::LazyLocator,
    features::{
        events::{EventEmitterExt, PluginEvent, SharedEventEmitter},
        instance::InstanceWatcherService,
        settings::{
            self, LocationInfo, Settings, SettingsStorage,
            infra::{FsSettingsStorage, SqliteSettingsStorage},
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
        launcher_dir: PathBuf,
        metadata_dir: PathBuf,
        event_emitter: SharedEventEmitter,
        sqlite_pool: sqlx::SqlitePool,
    ) -> crate::Result<()> {
        LAUNCHER_STATE
            .get_or_try_init(|| {
                Self::initialize(launcher_dir, metadata_dir, event_emitter, sqlite_pool)
            })
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
        launcher_dir: PathBuf,
        metadata_dir: PathBuf,
        event_emitter: SharedEventEmitter,
        sqlite_pool: sqlx::SqlitePool,
    ) -> crate::Result<Arc<Self>> {
        sqlx::migrate!()
            .run(&sqlite_pool)
            .await
            .map_err(|err| crate::ErrorKind::CoreError(format!("Migration failed: {err}")))?;

        let settings_storage = SqliteSettingsStorage::new(sqlite_pool.clone());

        settings::infra::migrate_to_sqlite(
            &FsSettingsStorage::new(&launcher_dir),
            &settings_storage,
        )
        .await?;

        let settings = if let Ok(settings) = settings_storage.get().await {
            settings
        } else {
            let settings = Settings::from_dirs(launcher_dir.clone(), metadata_dir);
            settings_storage.upsert(settings).await?
        };

        let location_info = Arc::new(LocationInfo::new(
            settings.launcher_dir().to_path_buf(),
            settings.metadata_dir().to_path_buf(),
        ));

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
