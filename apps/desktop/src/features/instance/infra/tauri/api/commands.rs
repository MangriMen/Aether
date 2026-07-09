use aether_core::{
    core::app::AetherContainer,
    features::{
        instance::{
            ChangeContentState, ContentStateAction, ImportContent, InstanceFeature, RemoveContent,
        },
        process::ProcessFeature,
        settings::SettingsFeature,
    },
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use tauri::State;
use uuid::Uuid;

use crate::{
    FrontendResult,
    features::{
        instance::infra::{
            CapabilityEntryDto, ContentCompatibilityCheckParamsDto, ContentCompatibilityResultDto,
            ContentFileDto, ContentGetParamsDto, ContentInstallParamsDto, ContentItemDto,
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
async fn create(
    new_instance: NewInstanceDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<String> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    Ok(container
        .create_instance_use_case()
        .execute(new_instance.into())
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn import(
    import_instance: ImportInstanceDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    Ok(container
        .import_instance_use_case()
        .execute(import_instance.into())
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn list_importers() -> FrontendResult<Vec<CapabilityEntryDto<ImporterCapabilityMetadataDto>>>
{
    let container = AetherContainer::get();
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
async fn list() -> FrontendResult<Vec<InstanceDto>> {
    let container = AetherContainer::get();
    Ok(container
        .list_instances_use_case()
        .execute()
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(std::convert::Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn get(id: String) -> FrontendResult<InstanceDto> {
    let container = AetherContainer::get();
    Ok(container
        .get_instance_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn get_dir(id: String) -> FrontendResult<PathBuf> {
    let container = AetherContainer::get();
    Ok(container.location_info().instance_dir(&id))
}

#[tauri::command]
#[specta::specta]
async fn install(
    id: String,
    force: bool,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    Ok(container
        .instance_install_service()
        .execute(id, force)
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn update(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    Ok(container
        .update_instance_use_case()
        .execute(id)
        .await
        .map_err(aether_core::Error::from)?)
}

// #[tauri::command]
// #[specta::specta]
// async fn list_updaters() -> FrontendResult<Vec<CapabilityEntry<UpdaterCapabilityMetadata>>> {
//     todo!()
// }

#[tauri::command]
#[specta::specta]
async fn edit(
    id: String,
    edit_instance: EditInstanceDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<InstanceDto> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    Ok(container
        .edit_instance_use_case()
        .execute(id, edit_instance.into())
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
) -> FrontendResult<MinecraftProcessMetadataDto> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
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
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
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
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
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
async fn list_content(id: String) -> FrontendResult<HashMap<String, ContentFileDto>> {
    let container = AetherContainer::get();
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
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    Ok(container
        .install_content_use_case()
        .execute(payload.into())
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
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
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
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
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
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();
    Ok(container
        .remove_content_use_case()
        .execute(RemoveContent::multiple(id, content_paths))
        .await
        .map_err(aether_core::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn list_content_providers()
-> FrontendResult<Vec<CapabilityEntryDto<ContentProviderCapabilityMetadataDto>>> {
    let container = AetherContainer::get();
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
async fn search_content(payload: ContentSearchParamsDto) -> FrontendResult<ContentSearchResultDto> {
    let container = AetherContainer::get();
    Ok(container
        .search_content_use_case()
        .execute(payload.into())
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn check_compatibility(
    instance_ids: HashSet<String>,
    check_params: ContentCompatibilityCheckParamsDto,
) -> FrontendResult<HashMap<String, ContentCompatibilityResultDto>> {
    let container = AetherContainer::get();
    Ok(container
        .check_content_compatibility_use_case()
        .execute(instance_ids, check_params.into())
        .await
        .map_err(aether_core::Error::from)?
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn get_content(params: ContentGetParamsDto) -> FrontendResult<ContentItemDto> {
    let container = AetherContainer::get();
    Ok(container
        .get_content_use_case()
        .execute(params.into())
        .await
        .map_err(aether_core::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn list_content_version(
    params: ContentListVersionParamsDto,
) -> FrontendResult<Vec<ContentVersionDto>> {
    let container = AetherContainer::get();
    Ok(container
        .list_content_versions_use_case()
        .execute(params.into())
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
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let container = AetherContainer::get();

    container
        .edit_instance_icon_use_case()
        .execute(edit_instance_icon.into())
        .await
        .map_err(aether_core::Error::from)?;

    Ok(())
}
