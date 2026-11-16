use std::sync::Arc;

use async_trait::async_trait;

use crate::features::plugins::{
    PluginError, PluginProviderError, PluginReleaseListing, PluginSourceType,
};

use super::ports::ListPluginReleasesUseCasePort;
use super::release_archive::{fetch_release_listing, pick_release};
use super::{PluginProviderFactory, ReleaseArchiveCache};

/// Lists the releases of a plugin that can be installed. Asks the provider only, never a release
/// archive, so the picker can show before anything is downloaded.
pub(crate) struct ListPluginReleasesUseCase {
    provider_factory: Arc<PluginProviderFactory>,
    archive_cache: Arc<ReleaseArchiveCache>,
}

impl ListPluginReleasesUseCase {
    pub(crate) fn new(
        provider_factory: Arc<PluginProviderFactory>,
        archive_cache: Arc<ReleaseArchiveCache>,
    ) -> Self {
        Self {
            provider_factory,
            archive_cache,
        }
    }

    pub(crate) async fn execute(
        &self,
        source_type: PluginSourceType,
        identifier: &str,
    ) -> Result<PluginReleaseListing, PluginError> {
        let (_, listing) = fetch_release_listing(
            &self.provider_factory,
            &self.archive_cache,
            &source_type,
            identifier,
        )
        .await?;
        let default_tag = pick_release(&listing.releases, None)
            .map(|r| r.tag_name.clone())
            .ok_or(PluginProviderError::ProviderNoAssets { source_type })?;

        Ok(PluginReleaseListing {
            owner: listing.owner.clone(),
            repo: listing.repo.clone(),
            releases: listing.releases.clone(),
            default_tag,
        })
    }
}

#[async_trait]
impl ListPluginReleasesUseCasePort for ListPluginReleasesUseCase {
    async fn execute(
        &self,
        source_type: PluginSourceType,
        identifier: &str,
    ) -> Result<PluginReleaseListing, PluginError> {
        self.execute(source_type, identifier).await
    }
}
