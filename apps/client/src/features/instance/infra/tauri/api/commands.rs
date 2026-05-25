use aether_core::{
    core::{LauncherState, domain::LazyLocator},
    features::instance::app::EditInstanceIconUseCase,
    shared::{AssetsResolver, FileCache, FsAssetsStorage},
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};
use tauri::State;
use uuid::Uuid;

use crate::{
    FrontendResult,
    features::{
        instance::{
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

    Ok(aether_core::api::instance::create(new_instance.into())
        .await
        .map_err(crate::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn import(
    import_instance: ImportInstanceDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    Ok(aether_core::api::instance::import(import_instance.into())
        .await
        .map_err(crate::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn list_importers() -> FrontendResult<Vec<CapabilityEntryDto<ImporterCapabilityMetadataDto>>>
{
    Ok(aether_core::api::instance::list_importers()
        .await
        .map_err(crate::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn list() -> FrontendResult<Vec<InstanceDto>> {
    Ok(aether_core::api::instance::list()
        .await
        .map_err(crate::Error::from)?
        .into_iter()
        .map(std::convert::Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn get(id: String) -> FrontendResult<InstanceDto> {
    Ok(aether_core::api::instance::get(id)
        .await
        .map_err(crate::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn get_dir(id: String) -> FrontendResult<PathBuf> {
    Ok(aether_core::api::instance::get_dir(&id)
        .await
        .map_err(crate::Error::from)?)
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

    Ok(aether_core::api::instance::install(id, force)
        .await
        .map_err(crate::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn update(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    Ok(aether_core::api::instance::update(id)
        .await
        .map_err(crate::Error::from)?)
}

// #[tauri::command]
// #[specta::specta]
// async fn list_updaters() -> FrontendResult<Vec<CapabilityEntry<UpdaterCapabilityMetadata>>> {
//     // Ok(aether_core::api::instance::list().await.map_err(crate::Error::from)?)
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

    Ok(aether_core::api::instance::edit(id, edit_instance.into())
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

    Ok(aether_core::api::instance::remove(id)
        .await
        .map_err(crate::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn launch(
    id: String,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<MinecraftProcessMetadataDto> {
    let _guard = idempotency.lock_cmd(request_id)?;

    Ok(aether_core::api::instance::run(id)
        .await
        .map_err(crate::Error::from)?
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

    Ok(aether_core::api::process::kill(uuid)
        .await
        .map_err(crate::Error::from)?)
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

    Ok(aether_core::api::instance::import_contents(
        instance_id,
        content_type.into(),
        source_paths.iter().map(PathBuf::from).collect(),
    )
    .await
    .map_err(crate::Error::from)?)
}

#[tauri::command]
#[specta::specta]
async fn list_content(id: String) -> FrontendResult<HashMap<String, ContentFileDto>> {
    Ok(aether_core::api::instance::list_content(id)
        .await
        .map_err(crate::Error::from)?
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

    Ok(aether_core::api::instance::install_content(payload.into())
        .await
        .map_err(crate::Error::from)?)
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

    Ok(
        aether_core::api::instance::enable_contents(id, content_paths)
            .await
            .map_err(crate::Error::from)?,
    )
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

    Ok(
        aether_core::api::instance::disable_contents(id, content_paths)
            .await
            .map_err(crate::Error::from)?,
    )
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

    Ok(
        aether_core::api::instance::remove_contents(id, content_paths)
            .await
            .map_err(crate::Error::from)?,
    )
}

#[tauri::command]
#[specta::specta]
async fn list_content_providers()
-> FrontendResult<Vec<CapabilityEntryDto<ContentProviderCapabilityMetadataDto>>> {
    Ok(aether_core::api::instance::list_content_providers()
        .await
        .map_err(crate::Error::from)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn search_content(payload: ContentSearchParamsDto) -> FrontendResult<ContentSearchResultDto> {
    Ok(aether_core::api::instance::search_content(payload.into())
        .await
        .map_err(crate::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn check_compatibility(
    instance_ids: HashSet<String>,
    check_params: ContentCompatibilityCheckParamsDto,
) -> FrontendResult<HashMap<String, ContentCompatibilityResultDto>> {
    Ok(
        aether_core::api::instance::check_compatibility(instance_ids, check_params.into())
            .await
            .map_err(crate::Error::from)?
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect(),
    )
}

#[tauri::command]
#[specta::specta]
async fn get_content(params: ContentGetParamsDto) -> FrontendResult<ContentItemDto> {
    Ok(aether_core::api::instance::get_content(params.into())
        .await
        .map_err(crate::Error::from)?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn list_content_version(
    params: ContentListVersionParamsDto,
) -> FrontendResult<Vec<ContentVersionDto>> {
    Ok(
        aether_core::api::instance::list_content_versions(params.into())
            .await
            .map_err(crate::Error::from)?
            .into_iter()
            .map(Into::into)
            .collect(),
    )
}

#[tauri::command]
#[specta::specta]
async fn edit_icon(
    edit_instance_icon: EditInstanceIconDto,
    request_id: RequestId,
    idempotency: State<'_, IdempotencyManager>,
) -> FrontendResult<()> {
    let _guard = idempotency.lock_cmd(request_id)?;

    let lazy_locator = LazyLocator::get().await.map_err(crate::Error::from)?;
    let state = LauncherState::get().await.map_err(crate::Error::from)?;

    let assets_cache = Arc::new(FileCache::new(AssetsResolver::new(
        state.location_info.clone(),
    )));

    let assets_manager = Arc::new(FsAssetsStorage::new(assets_cache));

    EditInstanceIconUseCase::new(lazy_locator.get_instance_storage().await, assets_manager)
        .execute(edit_instance_icon.into())
        .await
        .map_err(aether_core::Error::from)
        .map_err(crate::Error::from)?;

    Ok(())
}
