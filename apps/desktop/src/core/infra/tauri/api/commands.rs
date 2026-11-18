use std::path::PathBuf;

use tauri::State;

use crate::{
    FrontendResult,
    core::{AppSettingsStorageState, LocationInfoState, RecreateWindowUseCase, WindowManagerState},
    shared::{
        self,
        commands::{APPLICATION_PLUGIN_NAME, application_commands},
    },
};

#[must_use]
pub fn init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new(APPLICATION_PLUGIN_NAME)
        .invoke_handler(application_commands!(tauri::generate_handler!))
        .build()
}

#[must_use]
pub fn get_specta_commands() -> tauri_specta::Commands<tauri::Wry> {
    application_commands!(tauri_specta::collect_commands!)
}

#[tauri::command]
#[specta::specta]
async fn reveal_in_explorer(
    location_info: State<'_, LocationInfoState>,
    path: String,
    exact: bool,
) -> FrontendResult<()> {
    let roots = vec![
        location_info.settings_dir().to_path_buf(),
        location_info.config_dir().to_path_buf(),
    ];
    Ok(shared::reveal_in_explorer_async(PathBuf::from(path), exact, Some(roots)).await?)
}

#[tauri::command]
#[specta::specta]
async fn recreate_window(
    app_settings_storage: State<'_, AppSettingsStorageState>,
    window_manager: State<'_, WindowManagerState<tauri::Wry>>,
) -> FrontendResult<()> {
    Ok(RecreateWindowUseCase::new(
        app_settings_storage.inner().clone(),
        window_manager.inner().clone(),
    )
    .execute()
    .await?)
}
