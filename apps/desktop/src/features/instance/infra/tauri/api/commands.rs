use aether_core::features::{
    game_log::GameLogFeature,
    instance::{
        ChangeContentState, ContentCompatibilityCheckParams, ContentGetParams,
        ContentInstallParams, ContentListVersionsParams, ContentManagementPort, ContentPath,
        ContentProviderPort, ContentSearchParams, ContentStateAction, EditInstanceIcon,
        ImportContent, ImportInstance, InstanceCrudPort, InstanceError, InstanceId,
        InstanceLifecyclePort, InstanceServicesPort, NewInstance, RemoveContent,
    },
    process::{ProcessError, ProcessFeature},
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use tauri::State;
use uuid::Uuid;

use crate::{
    FrontendResult,
    core::ContainerState,
    features::{
        instance::infra::{
            BrokenInstanceDto, CapabilityEntryDto, ContentCompatibilityCheckParamsDto,
            ContentCompatibilityResultDto, ContentFileDto, ContentGetParamsDto,
            ContentInstallParamsDto, ContentItemDto, ContentListVersionParamsDto,
            ContentProviderCapabilityMetadataDto, ContentSearchParamsDto, ContentSearchResultDto,
            ContentTypeDto, ContentVersionDto, EditInstanceDto, EditInstanceIconDto,
            GameLogEventDto, ImportInstanceDto, ImporterCapabilityMetadataDto, InstanceDto,
            InstanceEventDto, LogCursorDto, LogPageDto, LogSourceDto, NewInstanceDto,
        },
        process::MinecraftProcessMetadataDto,
    },
    shared::{
        IdempotencyManager, RequestId, TauriIdempotencyExt,
        commands::{INSTANCE_PLUGIN_NAME, instance_commands},
    },
};

#[must_use]
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new(INSTANCE_PLUGIN_NAME)
        .invoke_handler(instance_commands!(tauri::generate_handler!))
        .build()
}

#[must_use]
pub fn get_specta_commands<R: tauri::Runtime>() -> tauri_specta::Commands<R> {
    instance_commands!(tauri_specta::collect_commands!)
}

#[must_use]
pub fn get_specta_events() -> tauri_specta::Events {
    tauri_specta::collect_events![InstanceEventDto, GameLogEventDto]
}

#[tauri::command]
#[specta::specta]
async fn create(
    new_instance: NewInstanceDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<String> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();
    Ok(container
        .create_instance_use_case()
        .execute(
            NewInstance::try_from(new_instance).map_err(aether_core::Error::from)?,
            None,
        )
        .await
        .map_err(aether_core::Error::from)?
        .to_string())
}

#[tauri::command]
#[specta::specta]
async fn import(
    import_instance: ImportInstanceDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();
    Ok(container
        .import_instance_use_case()
        .execute(ImportInstance::try_from(import_instance).map_err(aether_core::Error::from)?)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn list_importers(
    container: State<'_, ContainerState>,
) -> FrontendResult<Vec<CapabilityEntryDto<ImporterCapabilityMetadataDto>>> {
    let container = container.0.clone();
    Ok(container
        .list_importers_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn list(container: State<'_, ContainerState>) -> FrontendResult<Vec<InstanceDto>> {
    let container = container.0.clone();
    Ok(container
        .list_instances_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(std::convert::Into::into)
        .collect())
}

/// The instances that are stored but cannot be read, with why.
#[tauri::command]
#[specta::specta]
async fn list_broken(
    container: State<'_, ContainerState>,
) -> FrontendResult<Vec<BrokenInstanceDto>> {
    let container = container.0.clone();
    Ok(container
        .list_broken_instances_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

/// Removes a broken instance by the id `list_broken` gave. Not an instance that is readable.
#[tauri::command]
#[specta::specta]
async fn remove_broken(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();
    Ok(container
        .remove_broken_instance_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn get(id: String, container: State<'_, ContainerState>) -> FrontendResult<InstanceDto> {
    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    Ok(container
        .get_instance_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn get_dir(id: String, container: State<'_, ContainerState>) -> FrontendResult<PathBuf> {
    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    Ok(container.location_info().instance_dir(&id))
}

#[tauri::command]
#[specta::specta]
async fn install(
    id: String,
    force: bool,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    Ok(container
        .instance_install_service()
        .execute(id, force, None)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn update(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    Ok(container
        .update_instance_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn edit(
    id: String,
    edit_instance: EditInstanceDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<InstanceDto> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    Ok(container
        .edit_instance_use_case()
        .execute(id, edit_instance.try_into()?)
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

    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    Ok(container
        .remove_instance_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn launch(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<MinecraftProcessMetadataDto> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    Ok(container
        .launch_instance_with_active_account_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn stop(
    uuid: Uuid,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();
    Ok(container
        .kill_process_use_case()
        .execute(uuid)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn import_contents(
    instance_id: String,
    content_type: ContentTypeDto,
    source_paths: Vec<String>,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let instance_id = InstanceId::try_from(instance_id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    Ok(container
        .import_content_use_case()
        .execute(ImportContent::new(
            instance_id,
            content_type.into(),
            source_paths.iter().map(PathBuf::from).collect(),
        ))
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn list_content(
    id: String,
    container: State<'_, ContainerState>,
) -> FrontendResult<HashMap<String, ContentFileDto>> {
    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    Ok(container
        .list_content_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn install_content(
    payload: ContentInstallParamsDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();
    Ok(container
        .install_content_use_case()
        .execute(ContentInstallParams::try_from(payload).map_err(aether_core::Error::from)?)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn enable_contents(
    id: String,
    content_paths: Vec<String>,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let content_paths = parse_content_paths(&content_paths)?;
    let container = container.0.clone();
    Ok(container
        .change_content_state_use_case()
        .execute(ChangeContentState::multiple(
            id,
            content_paths,
            ContentStateAction::Enable,
        ))
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn disable_contents(
    id: String,
    content_paths: Vec<String>,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let content_paths = parse_content_paths(&content_paths)?;
    let container = container.0.clone();
    Ok(container
        .change_content_state_use_case()
        .execute(ChangeContentState::multiple(
            id,
            content_paths,
            ContentStateAction::Disable,
        ))
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn remove_contents(
    id: String,
    content_paths: Vec<String>,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let content_paths = parse_content_paths(&content_paths)?;
    let container = container.0.clone();
    Ok(container
        .remove_content_use_case()
        .execute(RemoveContent::multiple(id, content_paths))
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn list_content_providers(
    container: State<'_, ContainerState>,
) -> FrontendResult<Vec<CapabilityEntryDto<ContentProviderCapabilityMetadataDto>>> {
    let container = container.0.clone();
    Ok(container
        .list_providers_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn search_content(
    payload: ContentSearchParamsDto,
    container: State<'_, ContainerState>,
) -> FrontendResult<ContentSearchResultDto> {
    let container = container.0.clone();
    Ok(container
        .search_content_use_case()
        .execute(ContentSearchParams::try_from(payload).map_err(aether_core::Error::from)?)
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn check_compatibility(
    instance_ids: HashSet<String>,
    check_params: ContentCompatibilityCheckParamsDto,
    container: State<'_, ContainerState>,
) -> FrontendResult<HashMap<String, ContentCompatibilityResultDto>> {
    let instance_ids = instance_ids
        .into_iter()
        .map(InstanceId::try_from)
        .collect::<Result<HashSet<_>, _>>()
        .map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    Ok(container
        .check_content_compatibility_use_case()
        .execute(
            instance_ids,
            ContentCompatibilityCheckParams::try_from(check_params)
                .map_err(aether_core::Error::from)?,
        )
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.into()))
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn get_content(
    params: ContentGetParamsDto,
    container: State<'_, ContainerState>,
) -> FrontendResult<ContentItemDto> {
    let container = container.0.clone();
    Ok(container
        .get_content_use_case()
        .execute(ContentGetParams::try_from(params).map_err(aether_core::Error::from)?)
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn list_content_version(
    params: ContentListVersionParamsDto,
    container: State<'_, ContainerState>,
) -> FrontendResult<Vec<ContentVersionDto>> {
    let container = container.0.clone();
    Ok(container
        .list_content_versions_use_case()
        .execute(ContentListVersionsParams::try_from(params).map_err(aether_core::Error::from)?)
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn edit_icon(
    edit_instance_icon: EditInstanceIconDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = container.0.clone();

    container
        .edit_instance_icon_use_case()
        .execute(EditInstanceIcon::try_from(edit_instance_icon).map_err(aether_core::Error::from)?)
        .await
        .map_err(aether_core::Error::from)?;

    Ok(())
}

/// The content paths a command received, checked once here: the use cases only ever hold
/// [`ContentPath`]s, so a path that is not a plain relative chain of names (or reaches into
/// `.metadata`) is refused before any use case runs.
fn parse_content_paths(content_paths: &[String]) -> Result<Vec<ContentPath>, aether_core::Error> {
    content_paths
        .iter()
        .map(|content_path| ContentPath::parse(content_path))
        .collect::<Result<_, _>>()
        .map_err(|err| aether_core::Error::from(InstanceError::from(err)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_content_paths_when_all_are_plain_relative_paths() {
        let parsed = parse_content_paths(&["mods/a.jar".into(), r"config\b.toml".into()]).unwrap();

        assert_eq!(parsed[0].as_str(), "mods/a.jar");
        assert_eq!(parsed[1].as_str(), "config/b.toml");
    }

    #[test]
    fn should_refuse_the_whole_list_when_one_content_path_escapes_the_instance() {
        for hostile in [
            "../x",
            "mods/../../x",
            "/etc/passwd",
            ".metadata/instance.json",
        ] {
            assert!(
                parse_content_paths(&["mods/a.jar".into(), hostile.into()]).is_err(),
                "`{hostile}` must be refused"
            );
        }
    }
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
        .map_err(ProcessError::from)
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
        .map_err(ProcessError::from)
        .map_err(aether_core::Error::from)?;
    Ok(page.into())
}
