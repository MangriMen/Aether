use aether_core::features::settings::{
    app::{EditDefaultInstanceSettings, EditSettings},
    DefaultInstanceSettings, Settings,
};
use tauri::{AppHandle, State};

use crate::{
    features::settings::{
        AppSettings, AppSettingsStorageState, EditAppSettings, EditAppSettingsUseCase,
        GetAppSettingsUseCase, WindowManagerState,
    },
    FrontendResult,
};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("settings")
        .invoke_handler(tauri::generate_handler![
            get,
            edit,
            get_max_ram,
            get_default_instance_settings,
            edit_default_instance_settings,
            get_app_settings,
            edit_app_settings,
        ])
        .build()
}

#[tauri::command]
async fn get() -> FrontendResult<Settings> {
    Ok(aether_core::api::settings::get().await?)
}

#[tauri::command]
async fn edit(edit_settings: EditSettings) -> FrontendResult<Settings> {
    Ok(aether_core::api::settings::edit(edit_settings).await?)
}

#[tauri::command]
async fn get_max_ram() -> FrontendResult<u64> {
    Ok(aether_core::shared::infra::get_total_memory())
}

#[tauri::command]
async fn get_default_instance_settings() -> FrontendResult<DefaultInstanceSettings> {
    Ok(aether_core::api::settings::get_default_instance_settings().await?)
}

#[tauri::command]
async fn edit_default_instance_settings(
    edit_settings: EditDefaultInstanceSettings,
) -> FrontendResult<DefaultInstanceSettings> {
    Ok(aether_core::api::settings::upsert_default_instance_settings(edit_settings).await?)
}

#[tauri::command]
async fn get_app_settings(
    app_settings_storage: State<'_, AppSettingsStorageState>,
) -> FrontendResult<AppSettings> {
    Ok(
        GetAppSettingsUseCase::new(app_settings_storage.inner().clone())
            .execute()
            .await
            .map_err(crate::Error::from)?,
    )
}

#[tauri::command]
async fn edit_app_settings<R: tauri::Runtime>(
    _app_handle: AppHandle<R>,
    app_settings_storage: State<'_, AppSettingsStorageState>,
    window_manager: State<'_, WindowManagerState<R>>,
    edit_app_settings: EditAppSettings,
) -> FrontendResult<AppSettings> {
    Ok(EditAppSettingsUseCase::new(
        app_settings_storage.inner().clone(),
        window_manager.inner().clone(),
    )
    .execute(edit_app_settings)
    .await
    .map_err(crate::Error::from)?)
}
