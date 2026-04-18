use aether_core::{core::domain::LazyLocator, features::events::ListProgressBarsUseCase};

use crate::{
    commands::{events_commands, EVENTS_PLUGIN_NAME},
    features::events::{ProgressBarDto, ProgressEventDto, WarningEventDto},
    FrontendResult,
};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(EVENTS_PLUGIN_NAME)
        .invoke_handler(events_commands!(tauri::generate_handler!))
        .build()
}

pub fn get_specta_commands<R: tauri::Runtime>() -> tauri_specta::Commands<R> {
    events_commands!(tauri_specta::collect_commands!)
}

pub fn get_specta_events() -> tauri_specta::Events {
    tauri_specta::collect_events![ProgressEventDto, WarningEventDto]
}

#[tauri::command]
#[specta::specta]
pub async fn list_progress_bars() -> FrontendResult<Vec<ProgressBarDto>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ListProgressBarsUseCase::new(lazy_locator.get_progress_bar_storage().await)
            .execute()
            .await
            .into_iter()
            .map(Into::into)
            .collect(),
    )
}
