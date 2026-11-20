use crate::{
    FrontendResult,
    core::{ContainerState, UpdateServiceState},
    features::update::{CheckForUpdatesUseCase, InstallUpdateUseCase},
    shared::{
        IdempotencyManager, RequestId, TauriIdempotencyExt,
        commands::{UPDATE_PLUGIN_NAME, update_commands},
    },
};

use aether_core::features::process::ProcessFeature;
use tauri::State;

use super::UpdateStatusDto;

#[must_use]
pub fn init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new(UPDATE_PLUGIN_NAME)
        .invoke_handler(update_commands!(tauri::generate_handler!))
        .build()
}

#[must_use]
pub fn get_specta_commands<R: tauri::Runtime>() -> tauri_specta::Commands<R> {
    update_commands!(tauri_specta::collect_commands!)
}

#[tauri::command]
#[specta::specta]
pub async fn check_for_updates(
    update_service: State<'_, UpdateServiceState<tauri::Wry>>,
) -> FrontendResult<Option<UpdateStatusDto>> {
    Ok(CheckForUpdatesUseCase::new(update_service.inner().clone())
        .execute()
        .await
        .map_err(crate::Error::from)?
        .map(Into::into))
}

#[tauri::command]
#[specta::specta]
pub async fn install_update(
    update_service: State<'_, UpdateServiceState<tauri::Wry>>,
    container: State<'_, ContainerState>,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    Ok(InstallUpdateUseCase::new(
        update_service.inner().clone(),
        container.0.running_game_service(),
    )
    .execute()
    .await
    .map_err(crate::Error::from)?)
}
