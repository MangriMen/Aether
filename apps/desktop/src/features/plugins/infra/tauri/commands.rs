use std::path::{Path, PathBuf};

use aether_core::core::LazyLocator;
use tauri::State;

use crate::FrontendResult;
use crate::features::plugins::{
    EditPluginSettingsDto, PluginDto, PluginEventDto, PluginSettingsDto,
};
use crate::shared::reveal_in_explorer::infra::reveal_in_explorer;
use crate::shared::{
    IdempotencyManager, RequestId, TauriIdempotencyExt,
    commands::{PLUGIN_PLUGIN_NAME, plugin_commands},
};

#[must_use]
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(PLUGIN_PLUGIN_NAME)
        .invoke_handler(plugin_commands!(tauri::generate_handler!))
        .build()
}

#[must_use]
pub fn get_specta_commands<R: tauri::Runtime>() -> tauri_specta::Commands<R> {
    plugin_commands!(tauri_specta::collect_commands!)
}

#[must_use]
pub fn get_specta_events() -> tauri_specta::Events {
    tauri_specta::collect_events![PluginEventDto]
}

#[tauri::command]
#[specta::specta]
async fn import(
    paths: Vec<PathBuf>,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    Ok(aether_core::api::plugin::import(paths)
        .await
        .map_err(crate::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn sync(
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    Ok(aether_core::api::plugin::sync()
        .await
        .map_err(crate::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn list() -> FrontendResult<Vec<PluginDto>> {
    Ok(aether_core::api::plugin::list()
        .await
        .map_err(crate::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn get(id: String) -> FrontendResult<PluginDto> {
    Ok(aether_core::api::plugin::get(id)
        .await
        .map_err(crate::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn remove(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    Ok(aether_core::api::plugin::remove(id)
        .await
        .map_err(crate::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn enable(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    Ok(aether_core::api::plugin::enable(id)
        .await
        .map_err(crate::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn force_enable(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    Ok(aether_core::api::plugin::force_enable(id)
        .await
        .map_err(crate::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn disable(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    Ok(aether_core::api::plugin::disable(id)
        .await
        .map_err(crate::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn call(
    id: String,
    data: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    Ok(aether_core::api::plugin::call(id, data)
        .await
        .map_err(crate::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn get_settings(id: String) -> FrontendResult<Option<PluginSettingsDto>> {
    Ok(aether_core::api::plugin::get_settings(id)
        .await
        .map_err(crate::Error::from)?
        .map(Into::into))
}

#[tauri::command]
#[specta::specta]
async fn edit_settings(
    id: String,
    edit_settings: EditPluginSettingsDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    Ok(
        aether_core::api::plugin::edit_settings(id, edit_settings.into())
            .await
            .map_err(crate::Error::from)?,
    )
}

#[tauri::command]
#[specta::specta]
async fn open_plugins_folder() -> FrontendResult<()> {
    let locator = LazyLocator::get().await.map_err(crate::Error::from)?;
    let plugins_path = locator.location_info.plugins_dir();

    let path_to_open = if plugins_path.exists() {
        // If the folder exists, open it directly
        plugins_path
    } else {
        // If not, try to open the parent directory
        plugins_path
            .parent()
            .map_or_else(|| plugins_path.clone(), Path::to_path_buf)
    };

    Ok(reveal_in_explorer(&path_to_open, true)?)
}

#[tauri::command]
#[specta::specta]
async fn get_api_version() -> FrontendResult<semver::Version> {
    Ok(aether_core::api::plugin::get_api_version()
        .await
        .map_err(crate::Error::from)?)
}
