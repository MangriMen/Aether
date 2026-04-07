use std::{path::PathBuf, sync::Arc};

use tokio::sync::{OnceCell, Semaphore};

use crate::{
    core::domain::LazyLocator,
    features::{
        events::{EventEmitter, LauncherEvent},
        instance::InstanceWatcherService,
        settings::{infra::FsSettingsStorage, LocationInfo, Settings, SettingsStorage},
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
        app_handle: tauri::AppHandle,
    ) -> crate::Result<()> {
        LAUNCHER_STATE
            .get_or_try_init(|| Self::initialize(launcher_dir, metadata_dir, app_handle))
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

    #[tracing::instrument(skip(app_handle))]
    async fn initialize(
        launcher_dir: PathBuf,
        metadata_dir: PathBuf,
        app_handle: tauri::AppHandle,
    ) -> crate::Result<Arc<Self>> {
        let launcher_dir_path = launcher_dir.as_path();

        let settings_storage = FsSettingsStorage::new(launcher_dir_path);

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

        LazyLocator::init(state.clone(), app_handle).await?;

        let lazy_locator = LazyLocator::get().await?;
        lazy_locator
            .get_instance_watcher_service()
            .await?
            .watch_instances()
            .await?;

        let plugin_infra_listener = lazy_locator.get_plugin_infrastructure_listener().await;
        lazy_locator.get_event_emitter().await.listen::<_, ()>(
            LauncherEvent::Plugin.as_str(),
            move |data| {
                let value = plugin_infra_listener.clone();
                tokio::spawn(async move { value.on_plugin_event(data).await });
            },
        );

        log::info!("Service locator initialized");

        Ok(state)
    }
}
