use std::path::{Path, PathBuf};

use aether_core::features::{
    instance::PluginId,
    plugins::{PluginError, PluginSourceType, PluginsFeature, ProviderRelease},
};
use tauri::State;

use crate::{
    FrontendResult,
    core::ContainerState,
    features::plugins::{
        EditPluginSettingsDto, FolderAccessDto, HostAccessDto, PluginDto, PluginEventDto,
        PluginSettingsDto, PluginSourceDto, PluginSourceTypeDto, PluginUpdateInfoDto,
        ProviderPluginPreviewDto, SkippedPluginDirDto,
    },
    shared::reveal_in_explorer::infra::reveal_in_explorer,
    shared::{
        IdempotencyManager, RequestId, TauriIdempotencyExt,
        commands::{PLUGIN_PLUGIN_NAME, plugin_commands},
    },
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

/// An id that does not parse names no plugin, hence [`PluginError::NotFound`].
fn parse_id(id: String) -> Result<PluginId, aether_core::Error> {
    PluginId::try_from(id).map_err(|err| {
        PluginError::NotFound {
            plugin_id: err.value,
        }
        .into()
    })
}

#[tauri::command]
#[specta::specta]
async fn import(
    paths: Vec<PathBuf>,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();
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
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();
    Ok(container
        .sync_plugins_service()
        .execute()
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn list(container: State<'_, ContainerState>) -> FrontendResult<Vec<PluginDto>> {
    let container = container.0.clone();
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
async fn list_skipped_dirs(
    container: State<'_, ContainerState>,
) -> FrontendResult<Vec<SkippedPluginDirDto>> {
    let container = container.0.clone();
    Ok(container
        .list_skipped_plugin_dirs_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn get(id: String, container: State<'_, ContainerState>) -> FrontendResult<PluginDto> {
    let id = parse_id(id)?;
    let container = container.0.clone();
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
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = parse_id(id)?;
    let container = container.0.clone();
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
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = parse_id(id)?;
    let container = container.0.clone();
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
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = parse_id(id)?;
    let container = container.0.clone();
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
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = parse_id(id)?;
    let container = container.0.clone();
    Ok(container
        .disable_plugin_service()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?)
}

/// Restarts a running plugin, so that folders changed in its settings take effect. A plugin in
/// the middle of a call refuses with `Busy` and goes on as it was.
#[tauri::command]
#[specta::specta]
async fn reload(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = parse_id(id)?;
    let container = container.0.clone();
    Ok(container
        .reload_plugin_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn get_settings(
    id: String,
    container: State<'_, ContainerState>,
) -> FrontendResult<Option<PluginSettingsDto>> {
    let id = parse_id(id)?;
    let container = container.0.clone();
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
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = parse_id(id)?;
    let container = container.0.clone();
    Ok(container
        .edit_plugin_settings_use_case()
        .execute(id, edit_settings.into())
        .await
        .map_err(aether_core::Error::from)?)
}

/// Gives the plugin read access to the folder of a file the user named for it, restarting the
/// plugin if it runs. The UI asks the user first: with `dry_run` nothing changes, and the answer
/// says whether there is anything to ask (`needed`) or the plugin can already read the file.
#[tauri::command]
#[specta::specta]
async fn grant_folder_access(
    id: String,
    file_path: PathBuf,
    dry_run: bool,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<FolderAccessDto> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = parse_id(id)?;
    let container = container.0.clone();
    Ok(container
        .grant_plugin_folder_access_use_case()
        .execute(id, file_path, dry_run)
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

/// Lets the plugin reach the host of a URL the user gave it. No restart: the plugin's requests
/// read its hosts on every call. As with folders, the UI asks the user first, and `dry_run`
/// tells it whether there is anything to ask.
#[tauri::command]
#[specta::specta]
async fn grant_host_access(
    id: String,
    url: String,
    dry_run: bool,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<HostAccessDto> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = parse_id(id)?;
    let container = container.0.clone();
    Ok(container
        .grant_plugin_host_access_use_case()
        .execute(id, url, dry_run)
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn open_plugins_folder(container: State<'_, ContainerState>) -> FrontendResult<()> {
    let container = container.0.clone();
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
async fn get_api_version(container: State<'_, ContainerState>) -> FrontendResult<semver::Version> {
    let container = container.0.clone();
    Ok(container
        .get_plugin_api_version_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?)
}

// ── Provider factory commands ──

#[tauri::command]
#[specta::specta]
async fn get_available_providers(
    container: State<'_, ContainerState>,
) -> FrontendResult<Vec<PluginSourceTypeDto>> {
    let container = container.0.clone();
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
async fn get_plugin_source(
    id: String,
    container: State<'_, ContainerState>,
) -> FrontendResult<Option<PluginSourceDto>> {
    let id = parse_id(id)?;
    let container = container.0.clone();
    Ok(container
        .plugin_source_storage()
        .get(&id)
        .await
        .map_err(aether_core::Error::from)?
        .map(Into::into))
}

#[tauri::command]
#[specta::specta]
async fn check_for_updates(
    id: String,
    container: State<'_, ContainerState>,
) -> FrontendResult<PluginUpdateInfoDto> {
    let id = parse_id(id)?;
    let container = container.0.clone();
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
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = parse_id(id)?;
    let container = container.0.clone();
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
    container: State<'_, ContainerState>,
) -> FrontendResult<ProviderPluginPreviewDto> {
    let st: PluginSourceType = source_type.into();

    let container = container.0.clone();
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
#[allow(clippy::too_many_arguments)]
async fn install_plugin_from_provider(
    source_type: PluginSourceTypeDto,
    identifier: String,
    download_url: String,
    tag_name: String,
    version: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<String> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();
    Ok(container
        .install_plugin_from_provider_use_case()
        .execute(
            source_type.into(),
            &identifier,
            ProviderRelease {
                download_url,
                tag_name,
                version,
            },
        )
        .await
        .map_err(aether_core::Error::from)?
        .into())
}
