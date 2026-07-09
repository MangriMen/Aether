use std::path::{Path, PathBuf};

use aether_core::{
    core::app::AetherContainer,
    features::{
        plugins::{PluginSource, PluginSourceType, PluginsFeature, write_bytes_to_temp_file},
        settings::SettingsFeature,
    },
};
use tauri::State;

use crate::FrontendResult;
use crate::features::plugins::{
    EditPluginSettingsDto, PluginDto, PluginEventDto, PluginSettingsDto, PluginSourceDto,
    PluginSourceTypeDto, PluginUpdateInfoDto, ProviderPluginPreviewDto,
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

    let container = AetherContainer::get();
    Ok(container
        .import_plugins_use_case()
        .execute(paths)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn sync(
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    Ok(container
        .sync_plugins_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn list() -> FrontendResult<Vec<PluginDto>> {
    let container = AetherContainer::get();
    Ok(container
        .list_plugins_dto_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn get(id: String) -> FrontendResult<PluginDto> {
    let container = AetherContainer::get();
    Ok(container
        .get_plugin_dto_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?
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

    let container = AetherContainer::get();
    Ok(container
        .remove_plugin_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn enable(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    Ok(container
        .enable_plugin_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn force_enable(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    Ok(container
        .force_enable_plugin_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn disable(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    Ok(container
        .disable_plugin_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn call(
    id: String,
    _data: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    let reg = container.plugin_registry();
    let _plugin = reg.get(&id);
    log::debug!("Calling plugin {id:?}");
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn get_settings(id: String) -> FrontendResult<Option<PluginSettingsDto>> {
    let container = AetherContainer::get();
    Ok(container
        .get_plugin_settings_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?
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

    let container = AetherContainer::get();
    Ok(container
        .edit_plugin_settings_use_case()
        .execute(id, edit_settings.into())
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn open_plugins_folder() -> FrontendResult<()> {
    let container = AetherContainer::get();
    let plugins_path = container.location_info().plugins_dir();

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
    let container = AetherContainer::get();
    Ok(container
        .get_plugin_api_version_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?)
}

// ── Provider factory commands ──

#[tauri::command]
#[specta::specta]
async fn get_available_providers() -> FrontendResult<Vec<PluginSourceTypeDto>> {
    let container = AetherContainer::get();
    Ok(container
        .plugin_provider_factory()
        .list_source_types()
        .into_iter()
        .map(Into::into)
        .collect())
}

// ── Remote source commands ──

#[tauri::command]
#[specta::specta]
async fn get_plugin_source(id: String) -> FrontendResult<Option<PluginSourceDto>> {
    let container = AetherContainer::get();
    Ok(container
        .plugin_source_storage()
        .get(&id)
        .await
        .map_err(aether_core::Error::from)?
        .map(Into::into))
}

#[tauri::command]
#[specta::specta]
async fn check_for_updates(id: String) -> FrontendResult<PluginUpdateInfoDto> {
    let container = AetherContainer::get();
    Ok(container
        .check_for_plugin_updates_use_case()
        .execute(&id)
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn update_plugin(
    id: String,
    target_tag: Option<String>,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    Ok(container
        .update_plugin_use_case()
        .execute(&id, target_tag.as_deref())
        .await
        .map_err(aether_core::Error::from)?)
}

// ── Generic provider-based commands ──

#[tauri::command]
#[specta::specta]
async fn preview_plugin_from_provider(
    source_type: PluginSourceTypeDto,
    identifier: String,
) -> FrontendResult<ProviderPluginPreviewDto> {
    let st: PluginSourceType = source_type.into();

    let container = AetherContainer::get();
    let factory = container.plugin_provider_factory();
    let provider = factory.get_provider(&st).ok_or_else(|| {
        crate::Error::from(aether_core::Error::from(aether_core::ErrorKind::CoreError(
            format!("No provider registered for: {st:?}"),
        )))
    })?;

    let normalized = provider.parse_identifier(&identifier).map_err(|e| {
        crate::Error::from(aether_core::Error::from(aether_core::ErrorKind::CoreError(
            e.to_string(),
        )))
    })?;

    Ok(provider
        .fetch_preview(&normalized)
        .await
        .map_err(|e| crate::Error::from(aether_core::Error::from(e)))?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn install_plugin_from_provider(
    source_type: PluginSourceTypeDto,
    identifier: String,
    download_url: String,
    tag_name: String,
    version: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<String> {
    let _guard = idempotency.lock_cmd(request_id)?;
    let st: PluginSourceType = source_type.into();

    let container = AetherContainer::get();
    let factory = container.plugin_provider_factory();
    let provider = factory.get_provider(&st).ok_or_else(|| {
        crate::Error::from(aether_core::Error::from(aether_core::ErrorKind::CoreError(
            format!("No provider registered for: {st:?}"),
        )))
    })?;

    let normalized = provider.parse_identifier(&identifier).map_err(|e| {
        crate::Error::from(aether_core::Error::from(aether_core::ErrorKind::CoreError(
            e.to_string(),
        )))
    })?;

    let zip_bytes = provider
        .download_plugin(&download_url)
        .await
        .map_err(|e| crate::Error::from(aether_core::Error::from(e)))?;
    let temp_file = write_bytes_to_temp_file(&zip_bytes).map_err(aether_core::Error::from)?;
    let extracted = container
        .plugin_extractor()
        .extract(temp_file.path())
        .await
        .map_err(aether_core::Error::from)?;
    let plugin_id = extracted.plugin_id.clone();
    container
        .plugin_storage()
        .add(extracted)
        .await
        .map_err(aether_core::Error::from)?;

    let source = PluginSource::Remote {
        source_type: st,
        identifier: normalized,
        current_tag: tag_name,
        current_version: version,
    };
    container
        .plugin_source_storage()
        .save(&plugin_id, &source)
        .await
        .map_err(aether_core::Error::from)?;

    container
        .sync_plugins_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?;

    Ok(plugin_id)
}
