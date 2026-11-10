use aether_core::features::{game_log::GameLogFeature, instance::InstanceId};
use tauri::State;

use crate::{
    FrontendResult,
    core::ContainerState,
    features::game_log::infra::{GameLogEventDto, LogCursorDto, LogPageDto, LogSourceDto},
    shared::commands::{GAME_LOG_PLUGIN_NAME, game_log_commands},
};

#[must_use]
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(GAME_LOG_PLUGIN_NAME)
        .invoke_handler(game_log_commands!(tauri::generate_handler!))
        .build()
}

#[must_use]
pub fn get_specta_commands<R: tauri::Runtime>() -> tauri_specta::Commands<R> {
    game_log_commands!(tauri_specta::collect_commands!)
}

#[must_use]
pub fn get_specta_events() -> tauri_specta::Events {
    tauri_specta::collect_events![GameLogEventDto]
}

#[tauri::command]
#[specta::specta]
async fn list_logs(
    id: String,
    container: State<'_, ContainerState>,
) -> FrontendResult<Vec<LogSourceDto>> {
    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    let sources = container
        .list_game_log_sources_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?;
    Ok(sources.into_iter().map(Into::into).collect())
}

#[tauri::command]
#[specta::specta]
async fn read_log(
    id: String,
    source_id: String,
    cursor: LogCursorDto,
    max_bytes: u32,
    container: State<'_, ContainerState>,
) -> FrontendResult<LogPageDto> {
    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    let page = container
        .read_game_log_use_case()
        .execute(id, source_id, cursor.into(), max_bytes.into())
        .await
        .map_err(aether_core::Error::from)?;
    Ok(page.into())
}
