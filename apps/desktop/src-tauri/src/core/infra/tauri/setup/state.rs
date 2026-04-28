use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::{
    core::{
        AppSettingsStorageState, EventEmitterState, PreventExitState, TauriWindowManager,
        WindowManagerState,
    },
    features::{
        events::TauriEventEmitter, settings::FsAppSettingsStorage, update::TauriUpdateService,
    },
};

pub fn register_state<R: tauri::Runtime>(app_handle: &AppHandle<R>) {
    let app_settings_path = get_settings_path(app_handle);
    let app_settings_storage: AppSettingsStorageState =
        Arc::new(FsAppSettingsStorage::new(app_settings_path));

    let window_manager: WindowManagerState<R> =
        Arc::new(TauriWindowManager::new(app_handle.clone()));

    let event_emitter: EventEmitterState<R> = Arc::new(TauriEventEmitter::new(app_handle.clone()));

    let tauri_update_service = Arc::new(TauriUpdateService::new(
        app_handle.clone(),
        event_emitter.clone(),
    ));

    let prevent_exit_state = PreventExitState::new(false);

    app_handle.manage(app_settings_storage);
    app_handle.manage(window_manager);
    app_handle.manage(event_emitter);
    app_handle.manage(tauri_update_service);
    app_handle.manage(prevent_exit_state);
}

fn get_settings_path<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> std::path::PathBuf {
    app_handle
        .path()
        .app_config_dir()
        .expect("Failed to resolve app config directory")
        .join("aether_settings.json")
}
