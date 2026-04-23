#![allow(clippy::needless_pass_by_value)]
use std::{sync::Arc, time::Duration};

use aether_core::features::events::ProcessEvent;
use log::{error, warn};
use tauri::{App, AppHandle, Manager, WebviewWindow};

use crate::{
    core::{PreventExitStorage, build_main_window, instance_launch_listener},
    features::{
        events::{DualEventEmitterExt, EventEmitterState, TauriEventEmitter},
        settings::{
            AppSettings, AppSettingsStorage, AppSettingsStorageState, FsAppSettingsStorage,
            TauriWindowManager, WindowManager, WindowManagerState,
        },
        update::TauriUpdateService,
    },
};

const MAIN_WINDOW_VISIBLE_WATCH_DOG_TIMEOUT: Duration = Duration::from_secs(30);

pub fn init_app(app: &mut App) -> tauri::Result<()> {
    let app_handle = app.handle();

    let app_settings_path = get_settings_path(app_handle);

    let app_settings_storage = Arc::new(FsAppSettingsStorage::new(app_settings_path));
    let window_manager = Arc::new(TauriWindowManager::new(app_handle.clone()));
    let event_emitter = Arc::new(TauriEventEmitter::new(app_handle.clone()));

    let app_settings = tauri::async_runtime::block_on(async { app_settings_storage.get().await })
        .unwrap_or_default();

    let main_window = create_window(app_handle.clone(), app_settings)?;

    register_state(
        app_handle.clone(),
        app_settings_storage.clone(),
        window_manager.clone(),
        event_emitter.clone(),
    );

    let app_handle_task = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        setup_ui(app_settings, window_manager.clone())
            .await
            .unwrap_or_else(|err| error!("Failed to setup UI: {err}"));

        setup_listeners(app_handle_task.clone(), event_emitter.clone());

        main_window_visible_watch_dog(main_window).await;
    });

    Ok(())
}

fn register_state<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    app_settings_storage: AppSettingsStorageState,
    window_manager: WindowManagerState<R>,
    event_emitter: EventEmitterState<R>,
) {
    let tauri_update_service = Arc::new(TauriUpdateService::new(
        app_handle.clone(),
        event_emitter.clone(),
    ));

    let prevent_exit_state = std::sync::Mutex::new(PreventExitStorage::new(false));

    app_handle.manage(app_settings_storage);
    app_handle.manage(window_manager);
    app_handle.manage(event_emitter);
    app_handle.manage(tauri_update_service);
    app_handle.manage(prevent_exit_state);
}

fn create_window<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    settings: AppSettings,
) -> tauri::Result<WebviewWindow<R>> {
    build_main_window(app_handle.clone(), settings.transparent, false)
}

async fn setup_ui<R: tauri::Runtime>(
    settings: AppSettings,
    window_manager: WindowManagerState<R>,
) -> tauri::Result<()> {
    window_manager
        .apply_visual_effects(settings.window_effect)
        .await
        .unwrap_or_else(|err| warn!("Failed to apply visual effects: {err}"));

    Ok(())
}

// Prevent app exit when window closes after instance launched depends on app settings
fn setup_listeners(app_handle: AppHandle, event_emitter: EventEmitterState<tauri::Wry>) {
    event_emitter.on_core::<ProcessEvent, _>(move |e| {
        instance_launch_listener(app_handle.clone(), e);
    });
}

fn get_settings_path<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> std::path::PathBuf {
    app_handle
        .path()
        .app_config_dir()
        .expect("Failed to resolve app config directory")
        .join("aether_settings.json")
}

async fn main_window_visible_watch_dog<R: tauri::Runtime>(main_window: WebviewWindow<R>) {
    tokio::time::sleep(MAIN_WINDOW_VISIBLE_WATCH_DOG_TIMEOUT).await;

    if let Ok(false) = main_window.is_visible() {
        error!("Frontend initialization timeout. Emergency shutdown.");
        std::process::exit(1);
    }
}
