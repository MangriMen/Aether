use std::sync::Arc;

use aether_core::features::settings::LocationInfo;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};

use crate::{
    core::{
        AppSettingsStorageState, EventEmitterState, PreventExitState, TauriWindowManager,
        WindowManagerState,
    },
    features::{
        events::TauriEventEmitter,
        settings::{
            FsAppSettingsStorage, SqliteAppSettingsStorage, migrate_app_settings_to_sqlite,
        },
        update::TauriUpdateService,
    },
};

pub fn register_state<R: tauri::Runtime>(app_handle: &AppHandle<R>, pool: SqlitePool) {
    let app_settings_storage: AppSettingsStorageState =
        Arc::new(SqliteAppSettingsStorage::new(pool.clone()));

    let window_manager: WindowManagerState<R> =
        Arc::new(TauriWindowManager::new(app_handle.clone()));

    let event_emitter: EventEmitterState<R> = Arc::new(TauriEventEmitter::new(app_handle.clone()));

    let tauri_update_service = Arc::new(TauriUpdateService::new(
        app_handle.clone(),
        event_emitter.clone(),
    ));

    let prevent_exit_state = PreventExitState::new(false);

    app_handle.manage(pool);
    app_handle.manage(app_settings_storage);
    app_handle.manage(window_manager);
    app_handle.manage(event_emitter);
    app_handle.manage(tauri_update_service);
    app_handle.manage(prevent_exit_state);
}

pub async fn migrate<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    pool: SqlitePool,
) -> crate::Result<()> {
    let app_settings_path = get_settings_path(&app_handle);

    migrate_app_settings_to_sqlite(
        &FsAppSettingsStorage::new(app_settings_path),
        &SqliteAppSettingsStorage::new(pool.clone()),
        "migrated",
    )
    .await?;

    Ok(())
}

fn get_settings_path<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> std::path::PathBuf {
    app_handle
        .path()
        .app_config_dir()
        .expect("Failed to resolve app config directory")
        .join("aether_settings.json")
}

pub fn create_location_info<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> LocationInfo {
    let launcher_dir = app_handle
        .path()
        .app_config_dir()
        .expect("Failed to resolve app config directory");

    LocationInfo::new(launcher_dir.clone(), launcher_dir.clone())
}
