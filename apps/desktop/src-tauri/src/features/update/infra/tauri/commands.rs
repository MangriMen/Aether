use crate::{
    FrontendResult,
    commands::{UPDATE_PLUGIN_NAME, update_commands},
    core::UpdateServiceState,
    features::update::{CheckForUpdatesUseCase, InstallUpdateUseCase},
};

use tauri::State;

use super::UpdateStatusDto;

#[must_use]
pub fn init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new(UPDATE_PLUGIN_NAME)
        .invoke_handler(update_commands!(tauri::generate_handler!))
        .build()
}

#[must_use]
pub fn get_specta_commands() -> tauri_specta::Commands<tauri::Wry> {
    update_commands!(tauri_specta::collect_commands!)
}

#[tauri::command]
#[specta::specta]
pub async fn check_for_updates(
    update_service: State<'_, UpdateServiceState<tauri::Wry>>,
) -> FrontendResult<Option<UpdateStatusDto>, String> {
    Ok(CheckForUpdatesUseCase::new(update_service.inner().clone())
        .execute()
        .await?
        .map(Into::into))
}

#[tauri::command]
#[specta::specta]
pub async fn install_update(
    update_service: State<'_, UpdateServiceState<tauri::Wry>>,
) -> FrontendResult<(), String> {
    InstallUpdateUseCase::new(update_service.inner().clone())
        .execute()
        .await
}
