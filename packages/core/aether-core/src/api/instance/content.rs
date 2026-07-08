use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use dashmap::DashMap;

use crate::{
    core::app::AetherContainer,
    features::instance::{
        ChangeContentState, ContentCompatibilityCheckParams, ContentCompatibilityResult,
        ContentFile, ContentGetParams, ContentInstallParams, ContentItem,
        ContentListVersionsParams, ContentProviderCapabilityMetadata, ContentSearchParams,
        ContentSearchResult, ContentStateAction, ContentType, ContentVersion, ImportContent,
        InstanceFeature, RemoveContent,
    },
    shared::capability::domain::CapabilityEntry,
};

pub async fn list_content(instance_id: String) -> crate::Result<DashMap<String, ContentFile>> {
    let container = AetherContainer::get();
    Ok(container
        .list_content_use_case()
        .execute(instance_id)
        .await?)
}

pub async fn remove_contents(instance_id: String, content_paths: Vec<String>) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .remove_content_use_case()
        .execute(RemoveContent::multiple(instance_id, content_paths))
        .await?)
}

pub async fn enable_contents(instance_id: String, content_paths: Vec<String>) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .change_content_state_use_case()
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
    let container = AetherContainer::get();
    Ok(container
        .change_content_state_use_case()
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
    let container = AetherContainer::get();
    Ok(container
        .import_content_use_case()
        .execute(ImportContent::new(instance_id, content_type, source_paths))
        .await?)
}

pub async fn list_content_providers()
-> crate::Result<Vec<CapabilityEntry<ContentProviderCapabilityMetadata>>> {
    let container = AetherContainer::get();
    Ok(container.list_providers_use_case().execute().await?)
}

pub async fn search_content(
    search_params: ContentSearchParams,
) -> crate::Result<ContentSearchResult> {
    let container = AetherContainer::get();
    Ok(container
        .search_content_use_case()
        .execute(search_params)
        .await?)
}

pub async fn install_content(install_params: ContentInstallParams) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .install_content_use_case()
        .execute(install_params)
        .await?)
}

pub async fn check_compatibility(
    instance_ids: HashSet<String>,
    check_params: ContentCompatibilityCheckParams,
) -> crate::Result<HashMap<String, ContentCompatibilityResult>> {
    let container = AetherContainer::get();
    Ok(container
        .check_content_compatibility_use_case()
        .execute(instance_ids, check_params)
        .await?)
}

pub async fn get_content(params: ContentGetParams) -> crate::Result<ContentItem> {
    let container = AetherContainer::get();
    Ok(container.get_content_use_case().execute(params).await?)
}

pub async fn list_content_versions(
    params: ContentListVersionsParams,
) -> crate::Result<Vec<ContentVersion>> {
    let container = AetherContainer::get();
    Ok(container
        .list_content_versions_use_case()
        .execute(params)
        .await?)
}
