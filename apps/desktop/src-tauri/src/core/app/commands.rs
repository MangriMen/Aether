use std::path::Path;

use aether_core::core::LauncherState;
use tauri::{AppHandle, State};

use crate::{
    FrontendResult,
    commands::{APPLICATION_PLUGIN_NAME, application_commands},
    features::events::EventEmitterState,
    shared,
};

pub fn init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new(APPLICATION_PLUGIN_NAME)
        .invoke_handler(application_commands!(tauri::generate_handler!))
        .build()
}

pub fn get_specta_commands() -> tauri_specta::Commands<tauri::Wry> {
    application_commands!(tauri_specta::collect_commands!)
}

#[tauri::command]
#[specta::specta]
pub async fn initialize_state(
    app: AppHandle,
    event_emitter: State<'_, EventEmitterState<tauri::Wry>>,
) -> FrontendResult<()> {
    if LauncherState::initialized().await {
        return Ok(());
    }

    let launcher_dir = shared::get_app_dir(&app);
    LauncherState::init(
        launcher_dir.clone(),
        launcher_dir,
        event_emitter.inner().clone(),
    )
    .await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn initialize_plugins() -> FrontendResult<()> {
    aether_core::api::plugin::sync().await?;
    load_enabled_plugins().await?;
    Ok(())
}

pub async fn load_enabled_plugins() -> FrontendResult<()> {
    let settings = aether_core::api::settings::get().await?;

    for plugin_id in settings.enabled_plugins().iter() {
        if let Err(e) = aether_core::api::plugin::enable(plugin_id.to_string()).await {
            log::error!("Failed to load plugin {}: {}", plugin_id, e);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn reveal_in_explorer(path: String, exact: bool) -> FrontendResult<()> {
    Ok(shared::reveal_in_explorer(Path::new(&path), exact)?)
}
