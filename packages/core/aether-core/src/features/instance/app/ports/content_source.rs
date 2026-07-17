use std::collections::HashMap;

use async_trait::async_trait;

use crate::features::instance::domain::{
    ContentItem, ContentSearchParams, ContentSearchResult, ContentSourceCapabilityMetadata,
    ContentVersion, Instance, InstanceError, VersionInfo,
};

/// Unified contract for content sources — replaces both `ContentProvider` and `Importer`.
///
/// A `ContentSource` is a **read-only data source** that provides:
/// - Content search & metadata
/// - Version information (including whether a version is a single asset or a modpack)
///
/// The source does NOT perform installation or unpacking — it only returns data.
/// Installation logic lives in the core's `PackLifecycleHandler` (for modpacks)
/// or the generic asset download pipeline (for atomic assets).
#[async_trait]
pub trait ContentSource: Send + Sync {
    /// Metadata about this source for UI display.
    fn metadata(&self) -> &ContentSourceCapabilityMetadata;

    /// Search for content items.
    async fn search(
        &self,
        params: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError>;

    /// Get full details for a specific content item.
    async fn get_content(&self, content_id: String) -> Result<ContentItem, InstanceError>;

    /// List available versions for a content item.
    async fn list_versions(&self, content_id: String)
    -> Result<Vec<ContentVersion>, InstanceError>;

    /// Resolve a specific version — returns either an asset download instruction
    /// or a modpack manifest payload.
    async fn get_version_info(
        &self,
        content_id: &str,
        version_id: &str,
    ) -> Result<VersionInfo, InstanceError>;

    /// Check compatibility of a content item against multiple instances.
    async fn check_compatibility(
        &self,
        instances: &[Instance],
        content_item: &crate::features::instance::app::ContentCompatibilityCheckParams,
    ) -> Result<
        HashMap<String, crate::features::instance::app::ContentCompatibilityResult>,
        InstanceError,
    >;
}
