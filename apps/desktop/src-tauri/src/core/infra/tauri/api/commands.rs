#![allow(clippy::needless_pass_by_value)]
use std::path::Path;

use tauri::{AppHandle, State};

use crate::{
    FrontendResult,
    commands::{APPLICATION_PLUGIN_NAME, application_commands},
    core::{EventEmitterState, InitializeLauncherUseCase, InitializePluginsUseCase},
    shared,
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
pub async fn initialize_state(
    app: AppHandle,
    event_emitter: State<'_, EventEmitterState<tauri::Wry>>,
) -> FrontendResult<()> {
    let launcher_dir = shared::get_app_dir(&app);

    Ok(
        InitializeLauncherUseCase::new(event_emitter.inner().clone())
            .execute(launcher_dir)
            .await?,
    )
}

#[tauri::command]
#[specta::specta]
pub async fn initialize_plugins() -> FrontendResult<()> {
    Ok(InitializePluginsUseCase::execute().await?)
}

#[tauri::command]
pub fn reveal_in_explorer(path: String, exact: bool) -> FrontendResult<()> {
    Ok(shared::reveal_in_explorer(Path::new(&path), exact)?)
}
