use aether_core::features::events::EventsFeature;
use tauri::State;
use uuid::Uuid;

use crate::{
    FrontendResult,
    core::ContainerState,
    features::events::infra::{CacheEventDto, ProgressBarDto, ProgressEventDto, WarningEventDto},
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
    tauri_specta::collect_events![ProgressEventDto, WarningEventDto, CacheEventDto]
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

/// Call off the work of a bar, as the user asked. Nothing happens for a bar that is gone or
/// past the point where its work could stop: the frontend learns of it from the bar's events.
#[tauri::command]
#[specta::specta]
pub async fn cancel_progress(
    progress_bar_id: Uuid,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let container = container.0.clone();

    Ok(container
        .cancel_progress_use_case()
        .execute(progress_bar_id)
        .await
        .map_err(aether_core::Error::from)?)
}
