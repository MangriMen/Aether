use crate::features::plugins::{
    PluginError, PluginProviderError, PluginSourceType, ProviderPluginPreview, ProviderReleaseInfo,
};

use super::{DownloadedArchive, PluginProvider, download_to_temp_file};

/// The release asked for, from the provider's own listing: the one tagged `tag`, or with no tag
/// the first that is not a prerelease, else the first.
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

/// A release of a plugin, and its archive downloaded.
pub(super) struct ReleaseArchive {
    pub listing: ProviderPluginPreview,
    pub release: ProviderReleaseInfo,
    pub archive: DownloadedArchive,
}

/// Finds the release in the provider's listing and downloads its archive. The archive's address
/// is the provider's, never the caller's, so what is downloaded is the release of this very
/// repository. A release that is not listed, or that has no archive to install, is
/// `ProviderNoAssets`.
pub(super) async fn download_release_archive(
    provider: &dyn PluginProvider,
    source_type: &PluginSourceType,
    identifier: &str,
    tag: Option<&str>,
) -> Result<ReleaseArchive, PluginError> {
    let no_assets = || PluginProviderError::ProviderNoAssets {
        source_type: source_type.clone(),
    };

    let listing = provider.fetch_preview(identifier).await?;
    let release = pick_release(&listing.releases, tag)
        .filter(|r| !r.download_url.is_empty())
        .ok_or_else(no_assets)?
        .clone();
    let archive = download_to_temp_file(provider, &release.download_url).await?;

    Ok(ReleaseArchive {
        listing,
        release,
        archive,
    })
}
