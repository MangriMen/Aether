use aether_core::features::settings::{
    app::{EditDefaultInstanceSettings, EditSettings},
    DefaultInstanceSettings, Settings,
};
use tauri::{AppHandle, State};

use crate::{
    commands::{settings_commands, SETTINGS_PLUGIN_NAME},
    features::settings::{
        AppSettings, AppSettingsStorageState, EditAppSettingsDto, EditAppSettingsUseCase,
        GetAppSettingsUseCase, WindowManagerState,
    },
    FrontendResult,
};

pub fn init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new(SETTINGS_PLUGIN_NAME)
        .invoke_handler(settings_commands!(tauri::generate_handler!))
        .build()
}

pub fn get_specta_data() -> tauri_specta::Commands<tauri::Wry> {
    // settings_commands!(tauri_specta::collect_commands!)
    tauri_specta::collect_commands!(get_max_ram, get_app_settings, edit_app_settings)
}

#[tauri::command]
// #[specta::specta]
async fn get() -> FrontendResult<Settings> {
    Ok(aether_core::api::settings::get().await?)
}

#[tauri::command]
// #[specta::specta]
async fn edit(edit_settings: EditSettings) -> FrontendResult<Settings> {
    Ok(aether_core::api::settings::edit(edit_settings).await?)
}

#[tauri::command]
#[specta::specta]
async fn get_max_ram() -> FrontendResult<u64> {
    Ok(aether_core::shared::infra::get_total_memory())
}

#[tauri::command]
// #[specta::specta]
async fn get_default_instance_settings() -> FrontendResult<DefaultInstanceSettings> {
    Ok(aether_core::api::settings::get_default_instance_settings().await?)
}

#[tauri::command]
// #[specta::specta]
async fn edit_default_instance_settings(
    edit_settings: EditDefaultInstanceSettings,
) -> FrontendResult<DefaultInstanceSettings> {
    Ok(aether_core::api::settings::upsert_default_instance_settings(edit_settings).await?)
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
async fn edit_app_settings(
    _app_handle: AppHandle,
    app_settings_storage: State<'_, AppSettingsStorageState>,
    window_manager: State<'_, WindowManagerState<tauri::Wry>>,
    edit_app_settings: EditAppSettingsDto,
) -> FrontendResult<AppSettings> {
    Ok(EditAppSettingsUseCase::new(
        app_settings_storage.inner().clone(),
        window_manager.inner().clone(),
    )
    .execute(edit_app_settings)
    .await
    .map_err(crate::Error::from)?)
}
