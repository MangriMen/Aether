use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use dashmap::DashMap;

use crate::{
    core::LazyLocator,
    features::instance::{
        ContentFile, ContentInstallParams, ContentItem, ContentProviderCapabilityMetadata,
        ContentSearchParams, ContentSearchResult, ContentType, ContentVersion,
        app::{
            ChangeContentState, ChangeContentStateUseCase, CheckContentCompatibilityUseCase,
            ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentGetParams,
            ContentListVersionsParams, ContentStateAction, GetContentUseCase, ImportContent,
            ImportContentUseCase, InstallContentUseCase, ListContentUseCase,
            ListContentVersionsUseCase, ListProvidersUseCase, RemoveContent, RemoveContentUseCase,
            SearchContentUseCase,
        },
    },
    shared::CapabilityEntry,
};

pub async fn list_content(instance_id: String) -> crate::Result<DashMap<String, ContentFile>> {
    let locator = LazyLocator::get().await?;

    Ok(ListContentUseCase::new(
        locator.get_pack_storage().await,
        locator.location_info.clone(),
    )
    .execute(instance_id)
    .await?)
}

pub async fn remove_contents(instance_id: String, content_paths: Vec<String>) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(RemoveContentUseCase::new(
        locator.get_event_emitter().await,
        locator.get_pack_storage().await,
    )
    .execute(RemoveContent::multiple(instance_id, content_paths))
    .await?)
}

pub async fn enable_contents(instance_id: String, content_paths: Vec<String>) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(ChangeContentStateUseCase::new(
        locator.get_event_emitter().await,
        locator.location_info.clone(),
    )
    .execute(ChangeContentState::multiple(
        instance_id,
        content_paths,
        ContentStateAction::Enable,
    ))
    .await?)
}

pub async fn disable_contents(
    instance_id: String,
    content_paths: Vec<String>,
) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(ChangeContentStateUseCase::new(
        locator.get_event_emitter().await,
        locator.location_info.clone(),
    )
    .execute(ChangeContentState::multiple(
        instance_id,
        content_paths,
        ContentStateAction::Disable,
    ))
    .await?)
}

pub async fn import_contents(
    instance_id: String,
    content_type: ContentType,
    source_paths: Vec<PathBuf>,
) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(ImportContentUseCase::new(
        locator.get_event_emitter().await,
        locator.get_pack_storage().await,
        locator.location_info.clone(),
    )
    .execute(ImportContent::new(instance_id, content_type, source_paths))
    .await?)
}

pub async fn list_content_providers()
-> crate::Result<Vec<CapabilityEntry<ContentProviderCapabilityMetadata>>> {
    let locator = LazyLocator::get().await?;

    Ok(
        ListProvidersUseCase::new(locator.get_content_provider_registry().await)
            .execute()
            .await?,
    )
}

pub async fn search_content(
    search_params: ContentSearchParams,
) -> crate::Result<ContentSearchResult> {
    let locator = LazyLocator::get().await?;

    Ok(
        SearchContentUseCase::new(locator.get_content_provider_registry().await)
            .execute(search_params)
            .await?,
    )
}

pub async fn install_content(install_params: ContentInstallParams) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(InstallContentUseCase::new(
        locator.get_pack_storage().await,
        locator.get_content_provider_registry().await,
        locator.location_info.clone(),
    )
    .execute(install_params)
    .await?)
}

pub async fn check_compatibility(
    instance_ids: HashSet<String>,
    check_params: ContentCompatibilityCheckParams,
) -> crate::Result<HashMap<String, ContentCompatibilityResult>> {
    let locator = LazyLocator::get().await?;

    Ok(CheckContentCompatibilityUseCase::new(
        locator.get_content_provider_registry().await,
        locator.get_instance_storage().await,
    )
    .execute(instance_ids, check_params)
    .await?)
}

pub async fn get_content(params: ContentGetParams) -> crate::Result<ContentItem> {
    let locator = LazyLocator::get().await?;

    Ok(
        GetContentUseCase::new(locator.get_content_provider_registry().await)
            .execute(params)
            .await?,
    )
}

pub async fn list_content_versions(
    params: ContentListVersionsParams,
) -> crate::Result<Vec<ContentVersion>> {
    let locator = LazyLocator::get().await?;

    Ok(
        ListContentVersionsUseCase::new(locator.get_content_provider_registry().await)
            .execute(params)
            .await?,
    )
}
