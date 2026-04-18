use crate::{
    commands::{update_commands, UPDATE_PLUGIN_NAME},
    features::update::{
        CheckForUpdatesUseCase, InstallUpdateUseCase, UpdateProgress, UpdateServiceState,
        UpdateStatusDto,
    },
    FrontendResult,
};

use tauri::State;

pub fn init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new(UPDATE_PLUGIN_NAME)
        .invoke_handler(update_commands!(tauri::generate_handler!))
        .build()
}

pub fn get_specta_commands() -> tauri_specta::Commands<tauri::Wry> {
    update_commands!(tauri_specta::collect_commands!)
}

pub fn get_specta_events() -> tauri_specta::Events {
    tauri_specta::collect_events![UpdateProgress]
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
