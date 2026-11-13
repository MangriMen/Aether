use crate::features::plugins::{
    PluginError, PluginProviderError, PluginSourceType, ProviderFailure, ProviderPluginPreview,
    ProviderReleaseInfo,
};

use std::sync::Arc;

use super::{DownloadedArchive, PluginProviderFactory, ReleaseArchiveCache, download_to_temp_file};

/// With no tag: the first stable release, else the first.
pub(super) fn pick_release<'a>(
    releases: &'a [ProviderReleaseInfo],
    tag: Option<&str>,
) -> Option<&'a ProviderReleaseInfo> {
    match tag {
        Some(tag) => releases.iter().find(|r| r.tag_name == tag),
        None => releases
            .iter()
            .find(|r| !r.is_prerelease)
            .or_else(|| releases.first()),
    }
}

pub(super) struct ReleaseArchive {
    pub identifier: String,
    pub listing: ProviderPluginPreview,
    pub release: ProviderReleaseInfo,
    pub archive: Arc<DownloadedArchive>,
}

/// The archive URL comes from the provider's listing, never from the caller, so only this
/// repository's release is fetched. A release that is not listed or has no archive is
/// `ProviderNoAssets`.
pub(super) async fn download_release_archive(
    factory: &PluginProviderFactory,
    cache: &Arc<ReleaseArchiveCache>,
    source_type: &PluginSourceType,
    identifier: &str,
    tag: Option<&str>,
) -> Result<ReleaseArchive, PluginError> {
    let provider = factory.get_provider(source_type).ok_or_else(|| {
        PluginProviderError::ProviderFetchError {
            source_type: source_type.clone(),
            reason: ProviderFailure::NotRegistered,
        }
    })?;
    let identifier = provider.parse_identifier(identifier)?;
    let no_assets = || PluginProviderError::ProviderNoAssets {
        source_type: source_type.clone(),
    };

    let listing = provider.fetch_preview(&identifier).await?;
    let release = pick_release(&listing.releases, tag)
        .filter(|r| !r.download_url.is_empty())
        .ok_or_else(no_assets)?
        .clone();
    let archive = cache
        .get_or_download(&release.download_url, || {
            download_to_temp_file(provider, &release.download_url)
        })
        .await?;

    Ok(ReleaseArchive {
        identifier,
        listing,
        release,
        archive,
    })
}
