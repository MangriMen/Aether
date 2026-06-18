use std::sync::Arc;

use aether_core::{core::LazyLocator, features::settings::LocationInfo};
use log::error;
use sqlx::SqlitePool;
use tauri::{App, AppHandle, Manager};

use crate::{
    core::{
        AppSettingsStorageState, EventEmitterState, WindowLabel, WindowManager, WindowManagerState,
    },
    features::settings::{AppSettings, AppSettingsStorage},
};

use super::{
    listeners::setup_listeners, migrations::run_migrations, sqlite::create_pool,
    state::create_location_info, state::register_state, watchdog::run_window_is_visible_watch_dog,
};

pub fn initialize_app<R: tauri::Runtime>(app: &mut App<R>) {
    let app_handle = app.handle();

    let location_info = Arc::new(create_location_info(app_handle));

    let pool = tauri::async_runtime::block_on(async {
        let pool = create_pool(location_info.db_path())
            .await
            .expect("Failed to connect to DB");

        run_migrations(app_handle.clone(), pool.clone()).await;

        pool
    });

    register_state(app_handle, location_info.clone(), pool.clone());

    post_initialize(app_handle, location_info, pool);
}

fn post_initialize<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    location_info: Arc<LocationInfo>,
    pool: SqlitePool,
) {
    let app_settings_storage_state = app_handle.state::<AppSettingsStorageState>();
    let app_settings_storage = app_settings_storage_state.inner().clone();

    let window_manager_state = app_handle.state::<WindowManagerState<R>>();
    let event_emitter_state = app_handle.state::<EventEmitterState<R>>();

    let window_manager = window_manager_state.inner().clone();
    let event_emitter = event_emitter_state.inner().clone();

    tauri::async_runtime::spawn(async move {
        if let Err(err) = LazyLocator::init(location_info, event_emitter.clone(), pool).await {
            panic!("CRITICAL: Failed to initialize LazyLocator: {err}");
        }

        let main_window_label = WindowLabel::Main;

        let app_settings = app_settings_storage.get().await.unwrap_or_else(|err| {
            panic!("CRITICAL: Failed to get app settings: {err}");
        });

        window_manager
            .create_window(
                main_window_label,
                app_settings.transparent,
                app_settings.window_effect,
            )
            .await
            .unwrap_or_else(|err| {
                panic!("CRITICAL: Failed to create main window: {err}");
            });

        app_settings_storage
            .upsert(AppSettings {
                is_actual_transparent: app_settings.transparent,
                ..app_settings
            })
            .await
            .unwrap_or_else(|err| error!("Failed to set actual transparent: {err}"));

        setup_listeners(
            app_settings_storage.clone(),
            window_manager.clone(),
            event_emitter,
        );

        let main_window = window_manager
            .get_window(main_window_label)
            .expect("Main window handle disappeared after successful creation");

        run_window_is_visible_watch_dog(main_window).await;
    });
}
