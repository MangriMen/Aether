use aether_core::features::process::ProcessFeature;
use tauri::State;

use crate::{
    FrontendResult,
    core::ContainerState,
    features::{
        events::infra::WarningDto,
        process::{MinecraftProcessMetadataDto, ProcessEventDto},
    },
    shared::{
        IdempotencyManager, RequestId, TauriIdempotencyExt,
        commands::{PROCESS_PLUGIN_NAME, process_commands},
    },
};

#[must_use]
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(PROCESS_PLUGIN_NAME)
        .invoke_handler(process_commands!(tauri::generate_handler!))
        .build()
}

#[must_use]
pub fn get_specta_commands<R: tauri::Runtime>() -> tauri_specta::Commands<R> {
    process_commands!(tauri_specta::collect_commands!)
}

#[must_use]
pub fn get_specta_events() -> tauri_specta::Events {
    tauri_specta::collect_events![ProcessEventDto]
}

#[tauri::command]
#[specta::specta]
async fn list(
    container: State<'_, ContainerState>,
) -> FrontendResult<Vec<MinecraftProcessMetadataDto>> {
    let container = container.0.clone();
    Ok(container
        .list_process_metadata_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

/// The notices of skipped post-exit hooks that came before the window listened to events, to
/// be shown as warnings. Taken once: every later notice comes as a warning event.
#[tauri::command]
#[specta::specta]
async fn take_skipped_hook_notices(
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<Vec<WarningDto>> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();
    Ok(container
        .take_skipped_hook_notices_use_case()
        .execute()
        .await
        .into_iter()
        .map(Into::into)
        .collect())
}
