use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use dashmap::DashMap;

use crate::{
    core::{domain::LazyLocator, LauncherState},
    features::instance::{
        app::{
            ChangeContentState, ChangeContentStateUseCase, CheckContentCompatibilityUseCase,
            ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentGetParams,
            ContentListVersionParams, ContentStateAction, GetContentUseCase, ImportContent,
            ImportContentUseCase, InstallContentUseCase, ListContentUseCase,
            ListContentVersionUseCase, ListProvidersUseCase, RemoveContent, RemoveContentUseCase,
            SearchContentUseCase,
        },
        ContentFile, ContentInstallParams, ContentItem, ContentProviderCapabilityMetadata,
        ContentSearchParams, ContentSearchResult, ContentType, ContentVersion,
    },
    shared::CapabilityEntry,
};

pub async fn list_content(instance_id: String) -> crate::Result<DashMap<String, ContentFile>> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    Ok(ListContentUseCase::new(
        lazy_locator.get_pack_storage().await,
        state.location_info.clone(),
    )
    .execute(instance_id)
    .await?)
}

pub async fn remove_contents(instance_id: String, content_paths: Vec<String>) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(RemoveContentUseCase::new(
        lazy_locator.get_event_emitter().await,
        lazy_locator.get_pack_storage().await,
    )
    .execute(RemoveContent::multiple(instance_id, content_paths))
    .await?)
}

pub async fn enable_contents(instance_id: String, content_paths: Vec<String>) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    Ok(ChangeContentStateUseCase::new(
        lazy_locator.get_event_emitter().await,
        state.location_info.clone(),
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
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    Ok(ChangeContentStateUseCase::new(
        lazy_locator.get_event_emitter().await,
        state.location_info.clone(),
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
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    Ok(ImportContentUseCase::new(
        lazy_locator.get_event_emitter().await,
        lazy_locator.get_pack_storage().await,
        state.location_info.clone(),
    )
    .execute(ImportContent::new(instance_id, content_type, source_paths))
    .await?)
}

pub async fn list_content_providers(
) -> crate::Result<Vec<CapabilityEntry<ContentProviderCapabilityMetadata>>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ListProvidersUseCase::new(lazy_locator.get_content_provider_registry().await)
            .execute()
            .await?,
    )
}

pub async fn search_content(
    search_params: ContentSearchParams,
) -> crate::Result<ContentSearchResult> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        SearchContentUseCase::new(lazy_locator.get_content_provider_registry().await)
            .execute(search_params)
            .await?,
    )
}

pub async fn install_content(install_params: ContentInstallParams) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;
    let state = LauncherState::get().await?;

    Ok(InstallContentUseCase::new(
        lazy_locator.get_pack_storage().await,
        lazy_locator.get_content_provider_registry().await,
        state.location_info.clone(),
    )
    .execute(install_params)
    .await?)
}

pub async fn check_compatibility(
    instance_ids: HashSet<String>,
    check_params: ContentCompatibilityCheckParams,
) -> crate::Result<HashMap<String, ContentCompatibilityResult>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(CheckContentCompatibilityUseCase::new(
        lazy_locator.get_content_provider_registry().await,
        lazy_locator.get_instance_storage().await,
    )
    .execute(instance_ids, check_params)
    .await?)
}

pub async fn get_content(params: ContentGetParams) -> crate::Result<ContentItem> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetContentUseCase::new(lazy_locator.get_content_provider_registry().await)
            .execute(params)
            .await?,
    )
}

pub async fn list_content_version(
    params: ContentListVersionParams,
) -> crate::Result<Vec<ContentVersion>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ListContentVersionUseCase::new(lazy_locator.get_content_provider_registry().await)
            .execute(params)
            .await?,
    )
}
