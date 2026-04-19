use log::debug;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use uuid::Uuid;

use crate::{
    FrontendResult,
    commands::{INSTANCE_PLUGIN_NAME, instance_commands},
    features::{
        instance::{
            CapabilityEntryDto, ContentCompatibilityCheckParamsDto, ContentCompatibilityResultDto,
            ContentFileDto, ContentGetParamsDto, ContentInstallParamsDto, ContentItemDto,
            ContentListVersionParamsDto, ContentProviderCapabilityMetadataDto,
            ContentSearchParamsDto, ContentSearchResultDto, ContentTypeDto, ContentVersionDto,
            EditInstanceDto, ImportInstanceDto, ImporterCapabilityMetadataDto, InstanceDto,
            InstanceEventDto, NewInstanceDto,
        },
        process::MinecraftProcessMetadataDto,
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
async fn create(new_instance: NewInstanceDto) -> FrontendResult<()> {
    tokio::spawn(async move {
        if let Err(err) = aether_core::api::instance::create(new_instance.into()).await {
            debug!("{err:?}");
        }
    });
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn import(import_instance: ImportInstanceDto) -> FrontendResult<()> {
    Ok(aether_core::api::instance::import(import_instance.into()).await?)
}

#[tauri::command]
#[specta::specta]
async fn list_importers() -> FrontendResult<Vec<CapabilityEntryDto<ImporterCapabilityMetadataDto>>>
{
    Ok(aether_core::api::instance::list_importers()
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn list() -> FrontendResult<Vec<InstanceDto>> {
    Ok(aether_core::api::instance::list()
        .await?
        .into_iter()
        .map(std::convert::Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn get(id: String) -> FrontendResult<InstanceDto> {
    Ok(aether_core::api::instance::get(id).await?.into())
}

#[tauri::command]
#[specta::specta]
async fn get_dir(id: String) -> FrontendResult<PathBuf> {
    Ok(aether_core::api::instance::get_dir(&id).await?)
}

#[tauri::command]
#[specta::specta]
async fn install(id: String, force: bool) -> FrontendResult<()> {
    Ok(aether_core::api::instance::install(id, force).await?)
}

#[tauri::command]
#[specta::specta]
async fn update(id: String) -> FrontendResult<()> {
    Ok(aether_core::api::instance::update(id).await?)
}

// #[tauri::command]
// #[specta::specta]
// async fn list_updaters() -> FrontendResult<Vec<CapabilityEntry<UpdaterCapabilityMetadata>>> {
//     // Ok(aether_core::api::instance::list().await?)
//     todo!()
// }

#[tauri::command]
#[specta::specta]
async fn edit(id: String, edit_instance: EditInstanceDto) -> FrontendResult<InstanceDto> {
    Ok(aether_core::api::instance::edit(id, edit_instance.into())
        .await?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn remove(id: String) -> FrontendResult<()> {
    Ok(aether_core::api::instance::remove(id).await?)
}

#[tauri::command]
#[specta::specta]
async fn launch(id: String) -> FrontendResult<MinecraftProcessMetadataDto> {
    Ok(aether_core::api::instance::run(id).await?.into())
}

#[tauri::command]
#[specta::specta]
async fn stop(uuid: Uuid) -> FrontendResult<()> {
    Ok(aether_core::api::process::kill(uuid).await?)
}

#[tauri::command]
#[specta::specta]
async fn import_contents(
    instance_id: String,
    content_type: ContentTypeDto,
    source_paths: Vec<String>,
) -> FrontendResult<()> {
    Ok(aether_core::api::instance::import_contents(
        instance_id,
        content_type.into(),
        source_paths.iter().map(PathBuf::from).collect(),
    )
    .await?)
}

#[tauri::command]
#[specta::specta]
async fn list_content(id: String) -> FrontendResult<HashMap<String, ContentFileDto>> {
    Ok(aether_core::api::instance::list_content(id)
        .await?
        .into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn install_content(payload: ContentInstallParamsDto) -> FrontendResult<()> {
    Ok(aether_core::api::instance::install_content(payload.into()).await?)
}

#[tauri::command]
#[specta::specta]
async fn enable_contents(id: String, content_paths: Vec<String>) -> FrontendResult<()> {
    Ok(aether_core::api::instance::enable_contents(id, content_paths).await?)
}

#[tauri::command]
#[specta::specta]
async fn disable_contents(id: String, content_paths: Vec<String>) -> FrontendResult<()> {
    Ok(aether_core::api::instance::disable_contents(id, content_paths).await?)
}

#[tauri::command]
#[specta::specta]
async fn remove_contents(id: String, content_paths: Vec<String>) -> FrontendResult<()> {
    Ok(aether_core::api::instance::remove_contents(id, content_paths).await?)
}

#[tauri::command]
#[specta::specta]
async fn list_content_providers()
-> FrontendResult<Vec<CapabilityEntryDto<ContentProviderCapabilityMetadataDto>>> {
    Ok(aether_core::api::instance::list_content_providers()
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[tauri::command]
#[specta::specta]
async fn search_content(payload: ContentSearchParamsDto) -> FrontendResult<ContentSearchResultDto> {
    Ok(aether_core::api::instance::search_content(payload.into())
        .await?
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
            .await?
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect(),
    )
}

#[tauri::command]
#[specta::specta]
async fn get_content(params: ContentGetParamsDto) -> FrontendResult<ContentItemDto> {
    Ok(aether_core::api::instance::get_content(params.into())
        .await?
        .into())
}

#[tauri::command]
#[specta::specta]
async fn list_content_version(
    params: ContentListVersionParamsDto,
) -> FrontendResult<Vec<ContentVersionDto>> {
    Ok(
        aether_core::api::instance::list_content_version(params.into())
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    )
}
