use std::sync::Arc;

use aether_core::features::events::LauncherEvent;
use log::warn;
use tauri::{App, AppHandle, Listener, Manager};

use crate::{
    core::{build_main_window, instance_launch_listener, PreventExitStateInner},
    features::settings::{
        AppSettings, AppSettingsStorage, AppSettingsStorageState, FsAppSettingsStorage,
        TauriWindowManager, WindowManager, WindowManagerState,
    },
};

pub(super) fn init_app(app: &mut App) -> tauri::Result<()> {
    let app_handle = app.handle();

    let app_settings_path = get_settings_path(app_handle);
    let app_settings_storage = Arc::new(FsAppSettingsStorage::new(app_settings_path));

    let app_settings = tauri::async_runtime::block_on(async { app_settings_storage.get().await })
        .unwrap_or_default();

    let window_manager = Arc::new(TauriWindowManager::new(app_handle.clone()));

    init_app_state(
        app_handle.clone(),
        app_settings_storage.clone(),
        window_manager.clone(),
    );
    init_app_window(app_handle.clone(), &app_settings, window_manager.clone())?;
    init_instance_launch_listener(app_handle.clone());

    Ok(())
}

fn init_app_state<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    app_settings_storage: AppSettingsStorageState,
    window_manager: WindowManagerState<R>,
) {
    app_handle.manage(app_settings_storage);
    app_handle.manage(window_manager);
    app_handle.manage(std::sync::Mutex::new(PreventExitStateInner::new(false)));
}

fn init_app_window<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    settings: &AppSettings,
    window_manager: WindowManagerState<R>,
) -> tauri::Result<()> {
    build_main_window(app_handle.clone(), settings.transparent, false)?;

    tauri::async_runtime::block_on(async {
        window_manager
            .apply_visual_effects(settings.window_effect)
            .await
            .unwrap_or_else(|err| warn!("Failed to apply visual effects: {err}"));
    });
    Ok(())
}

// Prevent app exit when window closes after instance launched depends on app settings
fn init_instance_launch_listener(app_handle: AppHandle) {
    let app_handle_inner = app_handle.clone();
    app_handle.listen(LauncherEvent::Process.as_str(), move |e| {
        instance_launch_listener(app_handle_inner.clone(), e);
    });
}

fn get_settings_path<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> std::path::PathBuf {
    app_handle
        .path()
        .app_config_dir()
        .unwrap()
        .join("aether_settings.json")
}
