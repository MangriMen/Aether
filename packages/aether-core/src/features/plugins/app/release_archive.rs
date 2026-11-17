use crate::features::instance::PluginId;
use crate::features::plugins::{
    PluginError, PluginProvider, PluginProviderError, PluginSource, PluginSourceStorage,
    PluginSourceType, ProviderFailure, ProviderPluginPreview, ProviderReleaseInfo,
};

use std::sync::Arc;

use crate::shared::hash::HashAlgorithm;
use crate::shared::hash::infra::hash_file_hex_async;
use crate::shared::io::domain::IoError;

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

/// How a caller names a plugin to its provider.
#[derive(Debug, Clone, Copy)]
pub(super) enum ReleaseSource<'a> {
    /// As the user entered it, such as a repository URL: the provider normalizes it.
    Entered(&'a str),
    /// As the provider normalized it before: what an installed plugin's source records.
    Recorded(&'a str),
}

/// What a provider lists for a plugin, with the releases that have no archive left out: such a
/// release cannot be installed, so it must not be offered or picked by default. The same list,
/// held briefly, serves listing, previewing, installing and updating. Returns the identifier as
/// the provider normalizes it.
pub(super) async fn fetch_release_listing(
    factory: &PluginProviderFactory,
    cache: &ReleaseArchiveCache,
    source_type: &PluginSourceType,
    identifier: ReleaseSource<'_>,
) -> Result<(String, Arc<ProviderPluginPreview>), PluginError> {
    let provider = provider_of(factory, source_type)?;
    let identifier = match identifier {
        ReleaseSource::Entered(raw) => provider.parse_identifier(raw)?,
        ReleaseSource::Recorded(identifier) => identifier.to_owned(),
    };

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
    identifier: ReleaseSource<'_>,
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

/// The provider and identifier an installed plugin's releases come from, as its source records
/// them. A plugin with no remote source has none to update from.
pub(super) async fn recorded_source(
    storage: &dyn PluginSourceStorage,
    plugin_id: &PluginId,
) -> Result<(PluginSourceType, String), PluginError> {
    match storage.get(plugin_id).await? {
        Some(PluginSource::Remote {
            source_type,
            identifier,
            ..
        }) => Ok((source_type, identifier)),
        Some(PluginSource::Local) | None => Err(PluginProviderError::NoRemoteSource {
            plugin_id: plugin_id.clone(),
        }
        .into()),
    }
}

/// Refuses an archive that is not the one a preview showed: `expected_sha256` is the hex the
/// preview gave (any case). The file is hashed again, not trusted to be what it was when it was
/// downloaded: it may have sat in the cache since. Called before the archive is read.
pub(super) async fn ensure_previewed_archive(
    archive: &DownloadedArchive,
    expected_sha256: &str,
) -> Result<(), PluginError> {
    let sha256 = hash_file_hex_async(HashAlgorithm::Sha256, archive.path())
        .await
        .map_err(|e| PluginError::Storage(IoError::from(e)))?;
    if !sha256.eq_ignore_ascii_case(expected_sha256.trim()) {
        return Err(PluginProviderError::ArchiveChangedSincePreview.into());
    }
    Ok(())
}
