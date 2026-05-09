use tauri::{AppHandle, State};

use crate::{
    FrontendResult,
    core::{AppSettingsStorageState, WindowManagerState},
    features::settings::{EditAppSettingsUseCase, GetAppSettingsUseCase},
    shared::commands::{SETTINGS_PLUGIN_NAME, settings_commands},
};

use super::dtos::{
    AppSettingsDto, DefaultInstanceSettingsDto, EditAppSettingsDto, EditDefaultInstanceSettingsDto,
    EditSettingsDto, SettingsDto,
};

#[must_use]
pub fn init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new(SETTINGS_PLUGIN_NAME)
        .invoke_handler(settings_commands!(tauri::generate_handler!))
        .build()
}

#[must_use]
pub fn get_specta_commands() -> tauri_specta::Commands<tauri::Wry> {
    settings_commands!(tauri_specta::collect_commands!)
}

#[tauri::command]
#[specta::specta]
async fn get() -> FrontendResult<SettingsDto> {
    Ok(aether_core::api::settings::get()
        .await
        .map_err(crate::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn edit(edit_settings: EditSettingsDto) -> FrontendResult<SettingsDto> {
    Ok(aether_core::api::settings::edit(edit_settings.into())
        .await
        .map_err(crate::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn get_max_ram() -> FrontendResult<u64> {
    Ok(aether_core::shared::infra::get_total_memory())
}

#[tauri::command]
#[specta::specta]
async fn get_default_instance_settings() -> FrontendResult<DefaultInstanceSettingsDto> {
    Ok(aether_core::api::settings::get_default_instance_settings()
        .await
        .map_err(crate::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn edit_default_instance_settings(
    edit_settings: EditDefaultInstanceSettingsDto,
) -> FrontendResult<DefaultInstanceSettingsDto> {
    Ok(
        aether_core::api::settings::upsert_default_instance_settings(edit_settings.into())
            .await
            .map_err(crate::Error::from)?
            .into(),
    )
}

#[tauri::command]
#[specta::specta]
async fn get_app_settings(
    app_settings_storage: State<'_, AppSettingsStorageState>,
) -> FrontendResult<AppSettingsDto> {
    Ok(
        GetAppSettingsUseCase::new(app_settings_storage.inner().clone())
            .execute()
            .await
            .map_err(crate::Error::from)?
            .into(),
    )
}

#[tauri::command]
#[specta::specta]
async fn edit_app_settings(
    _app_handle: AppHandle,
    app_settings_storage: State<'_, AppSettingsStorageState>,
    window_manager: State<'_, WindowManagerState<tauri::Wry>>,
    edit_app_settings: EditAppSettingsDto,
) -> FrontendResult<AppSettingsDto> {
    Ok(EditAppSettingsUseCase::new(
        app_settings_storage.inner().clone(),
        window_manager.inner().clone(),
    )
    .execute(edit_app_settings.into())
    .await
    .map_err(crate::Error::from)?
    .into())
}
