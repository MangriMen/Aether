#![allow(clippy::needless_pass_by_value)]
use std::path::Path;

use tauri::State;

use crate::{
    FrontendResult,
    core::ContainerState,
    core::{
        AppSettingsStorageState, InitializePluginsUseCase, RecreateWindowUseCase,
        WindowManagerState,
    },
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
pub async fn wait_for_initialization(container: State<'_, ContainerState>) -> FrontendResult<()> {
    let _container = container.0.clone();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn initialize_plugins(container: State<'_, ContainerState>) -> FrontendResult<()> {
    Ok(InitializePluginsUseCase::execute(container.0.clone()).await?)
}

#[tauri::command]
pub fn reveal_in_explorer(path: String, exact: bool) -> FrontendResult<()> {
    Ok(shared::reveal_in_explorer(Path::new(&path), exact)?)
}

#[tauri::command]
#[specta::specta]
pub async fn recreate_window(
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
