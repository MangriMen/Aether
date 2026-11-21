use aether_core::features::instance::{
    ChangeContentState, ContentCompatibilityCheckParams, ContentGetParams, ContentGetVersionParams,
    ContentInstallParams, ContentListVersionsParams, ContentManagementPort, ContentPath,
    ContentProviderPort, ContentSearchParams, ContentStateAction, EditInstanceIcon, ImportContent,
    ImportInstance, InstanceCrudPort, InstanceError, InstanceId, InstanceLifecyclePort,
    InstanceServicesPort, NewInstance, RemoveContent,
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use tauri::State;

use crate::{
    FrontendResult,
    core::ContainerState,
    features::{
        instance::infra::{
            BrokenInstanceDto, CapabilityEntryDto, ContentCompatibilityCheckParamsDto,
            ContentCompatibilityResultDto, ContentFileDto, ContentGetParamsDto,
            ContentGetVersionParamsDto, ContentInstallParamsDto, ContentItemDto,
            ContentListVersionParamsDto, ContentProviderCapabilityMetadataDto,
            ContentSearchParamsDto, ContentSearchResultDto, ContentTypeDto, ContentVersionDto,
            EditInstanceDto, EditInstanceIconDto, ImportInstanceDto, ImporterCapabilityMetadataDto,
            InstanceDto, InstanceEventDto, NewInstanceDto,
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
    tauri_specta::collect_events![InstanceEventDto]
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
async fn get_dir(id: String, container: State<'_, ContainerState>) -> FrontendResult<PathBuf> {
    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    Ok(container.location_info().instance_dir(&id))
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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

/// The instances being launched, which `stop` calls off. The page asks it when it opens, for a
/// launch that began before it was reloaded; `LaunchStarted` and `LaunchEnded` tell the rest.
#[tauri::command]
#[specta::specta]
#[tracing::instrument(level = "debug", skip_all)]
async fn list_launching(container: State<'_, ContainerState>) -> FrontendResult<Vec<String>> {
    Ok(container
        .0
        .launch_cancellations()
        .launching()
        .into_iter()
        .map(|id| id.to_string())
        .collect())
}

/// Stops the instance: calls its launch off while it has no game yet, or ends its game. Nothing
/// to stop is no error.
#[tauri::command]
#[specta::specta]
#[tracing::instrument(level = "debug", skip_all)]
async fn stop(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
    container: State<'_, ContainerState>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let id = InstanceId::try_from(id).map_err(aether_core::Error::from)?;
    let container = container.0.clone();
    Ok(container
        .stop_instance_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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

/// The contents of several instances in one call, listed side by side. An instance whose
/// contents cannot be listed is left out of the answer, the others are not held back by it.
#[tauri::command]
#[specta::specta]
#[tracing::instrument(level = "debug", skip_all)]
async fn list_contents(
    ids: Vec<String>,
    container: State<'_, ContainerState>,
) -> FrontendResult<HashMap<String, HashMap<String, ContentFileDto>>> {
    let use_case = container.0.list_content_use_case();
    let mut listing = tokio::task::JoinSet::new();
    for id in ids {
        let Ok(instance_id) = InstanceId::try_from(id.clone()) else {
            tracing::warn!(instance_id = %id, "Could not list the contents of an instance: not a valid id");
            continue;
        };
        let use_case = use_case.clone();
        listing.spawn(async move { (id, use_case.execute(instance_id).await) });
    }

    let mut contents = HashMap::new();
    while let Some(done) = listing.join_next().await {
        let (id, result) = match done {
            Ok(done) => done,
            Err(err) => {
                tracing::warn!(error = %err, "Listing the contents of an instance did not finish");
                continue;
            }
        };
        match result {
            Ok(files) => {
                contents.insert(id, files.into_iter().map(|(k, v)| (k, v.into())).collect());
            }
            Err(err) => {
                tracing::warn!(instance_id = %id, error = %err, "Could not list the contents of the instance");
            }
        }
    }
    Ok(contents)
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
async fn get_content_version(
    params: ContentGetVersionParamsDto,
    container: State<'_, ContainerState>,
) -> FrontendResult<ContentVersionDto> {
    let container = container.0.clone();
    Ok(container
        .get_content_version_use_case()
        .execute(ContentGetVersionParams::try_from(params).map_err(aether_core::Error::from)?)
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
#[tracing::instrument(level = "debug", skip_all)]
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
