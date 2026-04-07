use std::collections::HashMap;

use async_trait::async_trait;

use crate::features::instance::{
    app::{ContentCompatibilityCheckParams, ContentCompatibilityResult},
    AtomicInstallParams, ContentFile, ContentItem, ContentProviderCapabilityMetadata,
    ContentSearchParams, ContentSearchResult, ContentVersion, DownloadedContent, Instance,
    InstanceError, ModpackInstallParams,
};

#[async_trait]
pub trait ContentProvider: Send + Sync {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata;

    async fn search(
        &self,
        search_content: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError>;

    async fn get_content(&self, content_id: String) -> Result<ContentItem, InstanceError>;

    async fn list_version(&self, content_id: String) -> Result<Vec<ContentVersion>, InstanceError>;

    async fn install_atomic(
        &self,
        install_params: &AtomicInstallParams,
    ) -> Result<DownloadedContent, InstanceError>;

    async fn install_modpack(
        &self,
        install_params: &ModpackInstallParams,
    ) -> Result<(String, Vec<ContentFile>), InstanceError>;

    async fn check_compatibility(
        &self,
        instance_ids: &[Instance],
        content_item: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError>;
}
