use log::error;
use tauri::{App, Manager};

use crate::{
    core::{
        AppSettingsStorageState, EventEmitterState, WindowLabel, WindowManager, WindowManagerState,
    },
    features::settings::{AppSettings, AppSettingsStorage},
};

use super::{
    listeners::setup_listeners, state::register_state, watchdog::run_window_is_visible_watch_dog,
};

pub fn init_app(app: &mut App) {
    let app_handle = app.handle();

    register_state(app_handle);

    let app_settings_storage_state = app_handle.state::<AppSettingsStorageState>();
    let app_settings_storage = app_settings_storage_state.inner().clone();

    let window_manager_state = app_handle.state::<WindowManagerState<tauri::Wry>>();
    let event_emitter_state = app_handle.state::<EventEmitterState<tauri::Wry>>();

    let window_manager = window_manager_state.inner().clone();
    let event_emitter = event_emitter_state.inner().clone();

    tauri::async_runtime::spawn(async move {
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
            event_emitter.clone(),
        );

        let main_window = window_manager
            .get_window(main_window_label)
            .expect("Main window handle disappeared after successful creation");

        run_window_is_visible_watch_dog(main_window).await;
    });
}
