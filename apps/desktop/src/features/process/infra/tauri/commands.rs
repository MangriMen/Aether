use aether_core::features::process::ProcessFeature;
use tauri::State;

use crate::{
    core::ContainerState,
    FrontendResult,
    features::process::{MinecraftProcessMetadataDto, ProcessEventDto},
    shared::commands::{PROCESS_PLUGIN_NAME, process_commands},
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

#[tauri::command]
#[specta::specta]
async fn get_by_instance_id(
    id: String,
    container: State<'_, ContainerState>,
) -> FrontendResult<Vec<MinecraftProcessMetadataDto>> {
    let container = container.0.clone();
    Ok(container
        .get_process_metadata_by_instance_id_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}
