use aether_core::features::settings::{
    DefaultInstanceSettings, EditDefaultInstanceSettings, Settings,
};
use tauri::{AppHandle, State};

use crate::{
    features::settings::{
        edit_app_settings_use_case, get_app_settings_use_case, AppSettings, AppSettingsState,
        EditAppSettings,
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
async fn edit(edit_settings: Settings) -> FrontendResult<Settings> {
    Ok(aether_core::api::settings::upsert(edit_settings).await?)
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
async fn get_app_settings(state: State<'_, AppSettingsState>) -> FrontendResult<AppSettings> {
    get_app_settings_use_case(state).await
}

#[tauri::command]
async fn edit_app_settings<R: tauri::Runtime>(
    app_handle: AppHandle<R>,
    state: State<'_, AppSettingsState>,
    edit_app_settings: EditAppSettings,
) -> FrontendResult<()> {
    edit_app_settings_use_case(app_handle, state, edit_app_settings).await
}
