use aether_core::features::events::EventsFeature;
use tauri::State;

use crate::{
    core::ContainerState,
    FrontendResult,
    features::events::infra::{ProgressBarDto, ProgressEventDto, WarningEventDto},
    shared::commands::{EVENTS_PLUGIN_NAME, events_commands},
};

#[must_use]
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(EVENTS_PLUGIN_NAME)
        .invoke_handler(events_commands!(tauri::generate_handler!))
        .build()
}

#[must_use]
pub fn get_specta_commands<R: tauri::Runtime>() -> tauri_specta::Commands<R> {
    events_commands!(tauri_specta::collect_commands!)
}

#[must_use]
pub fn get_specta_events() -> tauri_specta::Events {
    tauri_specta::collect_events![ProgressEventDto, WarningEventDto]
}

#[tauri::command]
#[specta::specta]
pub async fn list_progress_bars(
    container: State<'_, ContainerState>,
) -> FrontendResult<Vec<ProgressBarDto>> {
    let container = container.0.clone();

    Ok(container
        .list_progress_bars_use_case()
        .execute()
        .await
        .into_iter()
        .map(Into::into)
        .collect())
}
