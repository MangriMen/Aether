use crate::features::plugins::{
    PluginError, PluginProvider, PluginProviderError, PluginSourceType, ProviderFailure,
    ProviderPluginPreview, ProviderReleaseInfo,
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

fn provider_of<'a>(
    factory: &'a PluginProviderFactory,
    source_type: &PluginSourceType,
) -> Result<&'a dyn PluginProvider, PluginProviderError> {
    factory
        .get_provider(source_type)
        .ok_or_else(|| PluginProviderError::ProviderFetchError {
            source_type: source_type.clone(),
            reason: ProviderFailure::NotRegistered,
        })
}

pub(super) struct ReleaseArchive {
    pub identifier: String,
    pub release: ProviderReleaseInfo,
    pub archive: Arc<DownloadedArchive>,
}

/// What a provider lists for a plugin, with the releases that have no archive left out: such a
/// release cannot be installed, so it must not be offered or picked by default. The same list,
/// held briefly, serves listing, previewing and installing. Returns the identifier as the
/// provider normalizes it.
pub(super) async fn fetch_release_listing(
    factory: &PluginProviderFactory,
    cache: &ReleaseArchiveCache,
    source_type: &PluginSourceType,
    identifier: &str,
) -> Result<(String, Arc<ProviderPluginPreview>), PluginError> {
    let provider = provider_of(factory, source_type)?;
    let identifier = provider.parse_identifier(identifier)?;

    let listing = cache
        .get_or_fetch_listing(&format!("{source_type}:{identifier}"), || async {
            let mut listing = provider.fetch_preview(&identifier).await?;
            listing.releases.retain(|r| !r.download_url.is_empty());
            // Checked here so that an empty list is not kept: the author may fix the release.
            if listing.releases.is_empty() {
                return Err(PluginProviderError::ProviderNoAssets {
                    source_type: source_type.clone(),
                }
                .into());
            }
            Ok(listing)
        })
        .await?;

    Ok((identifier, listing))
}

/// The archive URL comes from the provider's listing, never from the caller, so only this
/// repository's release is fetched. A release that is not listed is `ProviderNoAssets`.
pub(super) async fn download_release_archive(
    factory: &PluginProviderFactory,
    cache: &Arc<ReleaseArchiveCache>,
    source_type: &PluginSourceType,
    identifier: &str,
    tag: &str,
) -> Result<ReleaseArchive, PluginError> {
    let (identifier, listing) =
        fetch_release_listing(factory, cache, source_type, identifier).await?;
    let release = pick_release(&listing.releases, Some(tag))
        .ok_or_else(|| PluginProviderError::ProviderNoAssets {
            source_type: source_type.clone(),
        })?
        .clone();
    let provider = provider_of(factory, source_type)?;
    let archive = cache
        .get_or_download(&release.download_url, || {
            download_to_temp_file(provider, &release.download_url)
        })
        .await?;

    Ok(ReleaseArchive {
        identifier,
        release,
        archive,
    })
}
