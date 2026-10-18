//! `download_to_cache` — a host-side download into the plugin's own `/cache`.
//!
//! Large content must not travel through the plugin's linear memory, so the host downloads it
//! to disk and hands back a path the plugin opens through WASI. Decided here, not by the plugin:
//!
//! * **Network policy** — exactly `http_get`'s, through the same code ([`super::http`]):
//!   `allowed_hosts`, public addresses only, pinned connections, every redirect hop vetted.
//! * **Transfer** — [`RequestClient::download_to_file`]: the download semaphore, retries, the
//!   hash checked while streaming, a temporary file renamed only once verified. Redirects are
//!   followed below that pipeline, so only the final body is hashed and a retry restarts the
//!   chain from the plugin's URL.
//! * **Name** — content-addressed, `/cache/downloads/<algorithm>/<hex digest>`, built from the
//!   validated hash, so nothing the plugin sends reaches a path unchecked.
//!
//! `/cache` is writable from the sandbox, so nothing found there is trusted: a cached file is
//! re-hashed before reuse, and the directories on the way must not be links out of the cache.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use aether_core_plugin_api::v0::{CachedFileDto, DownloadToCacheParamsDto};
use extism::{convert::Msgpack, host_fn};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};

use crate::{
    features::{events::ProgressService, plugins::infra::plugin_utils::PLUGIN_CACHE_MOUNT},
    shared::{
        fetch::FetchSemaphore,
        hash::{FileHash, HashAlgorithm, InvalidFileHash, infra::hash_file_hex_async},
        limits::{DOWNLOAD_BUDGET, DOWNLOAD_MAX_RETRIES, DOWNLOAD_TIMEOUT, PluginSandboxLimits},
        request_client::{
            BytesProgress, Request, RequestClient, RequestError, infra::ReqwestClient,
        },
    },
};

use super::{
    super::{super::mappers::to_extism_res, PluginContext, block_on},
    http::{
        AddressFilter, PluginHttpError, ProxySource, check_target, is_blocked_ip, policy_error,
        system_lookup, vetted_client,
    },
};

/// Subdirectory of the plugin cache the host downloads into.
const DOWNLOADS_DIR: &str = "downloads";

#[derive(Debug, thiserror::Error)]
pub enum PluginDownloadError {
    #[error("Invalid hash: {0}")]
    InvalidHash(#[from] InvalidFileHash),

    #[error(transparent)]
    Http(#[from] PluginHttpError),

    #[error("Download failed: {0}")]
    Request(#[from] RequestError),

    #[error("Plugin cache path `{}` cannot be used: {reason}", path.display())]
    UnusableCache { path: PathBuf, reason: String },

    #[error("Download did not finish within {} seconds", .0.as_secs())]
    OutOfTime(Duration),
}

impl From<PluginDownloadError> for crate::ErrorKind {
    fn from(value: PluginDownloadError) -> Self {
        Self::CoreError(value.to_string())
    }
}

fn unusable(path: &Path, reason: &dyn std::fmt::Display) -> PluginDownloadError {
    PluginDownloadError::UnusableCache {
        path: path.to_path_buf(),
        reason: reason.to_string(),
    }
}

/// What `download_to_cache` needs from the core, apart from `AetherContainer` so tests can run
/// the pipeline against a temporary directory and a loopback server.
pub(crate) struct DownloadContext<'a> {
    /// Host side of the plugin's `/cache` (`LocationInfo::plugin_cache_dir`).
    pub cache_dir: &'a Path,
    pub allowed_hosts: &'a [String],
    pub semaphore: Arc<FetchSemaphore>,
    pub progress_service: Arc<dyn ProgressService>,
    pub proxy: ProxySource,
    pub limits: PluginSandboxLimits,
    /// [`DOWNLOAD_BUDGET`] in production.
    pub budget: Duration,
}

/// The plugin-side path of the cache entry for `hash`.
fn plugin_path(hash: &FileHash) -> String {
    format!(
        "{PLUGIN_CACHE_MOUNT}/{DOWNLOADS_DIR}/{}/{}",
        hash.algorithm(),
        hash.hex()
    )
}

/// Create `<cache>/downloads/<algorithm>` and prove it is a real directory inside the cache.
///
/// The plugin can write to `/cache`, so it may have put a link at either level, and
/// `create_dir_all` would follow it and create directories wherever it points. The canonical
/// check at the end covers anything `symlink_metadata` would not report.
async fn prepare_download_dir(
    cache_dir: &Path,
    algorithm: HashAlgorithm,
) -> Result<PathBuf, PluginDownloadError> {
    // The cache root is the core's: it is the mount root, which the plugin cannot replace.
    crate::shared::io::infra::create_dir_all(cache_dir)
        .await
        .map_err(|err| unusable(cache_dir, &err))?;

    let mut dir = cache_dir.to_path_buf();
    for segment in [DOWNLOADS_DIR, algorithm.name()] {
        dir.push(segment);

        match tokio::fs::symlink_metadata(&dir).await {
            // `is_dir` on link metadata is false for a link, whatever it points to.
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => return Err(unusable(&dir, &"not a plain directory")),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                tokio::fs::create_dir(&dir)
                    .await
                    .map_err(|err| unusable(&dir, &err))?;
            }
            Err(err) => return Err(unusable(&dir, &err)),
        }
    }

    let canonical_root = crate::shared::io::infra::canonicalize(cache_dir)
        .map_err(|err| unusable(cache_dir, &err))?;
    let canonical_dir =
        crate::shared::io::infra::canonicalize(&dir).map_err(|err| unusable(&dir, &err))?;

    if !canonical_dir.starts_with(&canonical_root) {
        return Err(unusable(&dir, &"resolves outside the plugin cache"));
    }

    Ok(canonical_dir)
}

/// The size of a valid file already cached at `destination`, if there is one.
///
/// Anything else under that name is removed, so the download writes a fresh file instead of
/// writing through a link the plugin put there. A directory is refused, not deleted recursively.
async fn reuse_cached(
    destination: &Path,
    hash: &FileHash,
    max_bytes: u64,
) -> Result<Option<u64>, PluginDownloadError> {
    let metadata = match tokio::fs::symlink_metadata(destination).await {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(unusable(destination, &err)),
    };

    if metadata.is_dir() {
        return Err(unusable(destination, &"occupied by a directory"));
    }

    if metadata.is_file() && metadata.len() <= max_bytes {
        let actual = hash_file_hex_async(hash.algorithm(), destination)
            .await
            .map_err(|err| unusable(destination, &err))?;

        if hash.matches(&actual) {
            return Ok(Some(metadata.len()));
        }
    }

    tokio::fs::remove_file(destination)
        .await
        .map_err(|err| unusable(destination, &err))?;

    Ok(None)
}

/// The whole `download_to_cache` pipeline, parameterised by the address filter for the same
/// reason as `http_get`'s: tests need a loopback server without relaxing the real policy.
///
/// Cut off at `context.budget`. Nothing is left behind then: the download writes to a temporary
/// file that is deleted when the cut-off drops it.
async fn perform_download(
    params: DownloadToCacheParamsDto,
    context: &DownloadContext<'_>,
    is_blocked: AddressFilter,
) -> Result<CachedFileDto, PluginDownloadError> {
    tokio::time::timeout(
        context.budget,
        perform_download_unbounded(params, context, is_blocked),
    )
    .await
    .map_err(|_elapsed| PluginDownloadError::OutOfTime(context.budget))?
}

async fn perform_download_unbounded(
    params: DownloadToCacheParamsDto,
    context: &DownloadContext<'_>,
    is_blocked: AddressFilter,
) -> Result<CachedFileDto, PluginDownloadError> {
    // The digest becomes a file name, so it is validated before anything else happens.
    let hash = FileHash::from(params.hash).normalized()?;
    let (url, _) = check_target(&params.url, context.allowed_hosts)?;

    let dir = prepare_download_dir(context.cache_dir, hash.algorithm()).await?;
    let destination = dir.join(hash.hex());
    let path = plugin_path(&hash);

    if let Some(size) = reuse_cached(&destination, &hash, context.limits.max_download_bytes).await?
    {
        return Ok(CachedFileDto { path, size });
    }

    let network = PluginNetwork {
        allowed_hosts: context.allowed_hosts,
        semaphore: context.semaphore.clone(),
        progress_service: context.progress_service.clone(),
        proxy: &context.proxy,
        is_blocked,
        limits: context.limits,
    };
    let size = download_vetted(url.as_str(), Some(hash), &destination, &network).await?;

    Ok(CachedFileDto { path, size })
}

/// A plugin's network, as a download the host makes for it sees it.
pub(crate) struct PluginNetwork<'a> {
    pub allowed_hosts: &'a [String],
    pub semaphore: Arc<FetchSemaphore>,
    pub progress_service: Arc<dyn ProgressService>,
    pub proxy: &'a ProxySource,
    /// [`is_blocked_ip`] in production.
    pub is_blocked: AddressFilter,
    pub limits: PluginSandboxLimits,
}

/// Download `url` to `destination` under a plugin's network policy: `download_to_cache`
/// without its cache. Also used for the URLs a plugin puts into an install plan, which the
/// application's client would let redirect anywhere. On any failure nothing is left at
/// `destination`.
pub(crate) async fn download_vetted(
    url: &str,
    hash: Option<FileHash>,
    destination: &Path,
    network: &PluginNetwork<'_>,
) -> Result<u64, PluginDownloadError> {
    check_target(url, network.allowed_hosts)?;
    let client = vetted_download_client(network)?;

    download_with(
        &client,
        network.allowed_hosts,
        url,
        hash,
        destination,
        network.limits.max_download_bytes,
        None,
    )
    .await
}

/// The client [`download_vetted`] downloads through, for a caller with many files under one
/// policy. It holds `network`'s policy as it is now: a caller keeping it must build another
/// when the plugin's `allowed_hosts` change.
pub(crate) fn vetted_download_client(
    network: &PluginNetwork<'_>,
) -> Result<VettedDownloadClient, PluginDownloadError> {
    let retries = RetryTransientMiddleware::new_with_policy(
        ExponentialBackoff::builder().build_with_max_retries(DOWNLOAD_MAX_RETRIES),
    );
    let client = vetted_client(
        network.allowed_hosts,
        system_lookup,
        network.is_blocked,
        network.proxy,
        DOWNLOAD_TIMEOUT,
        Some(Arc::new(retries)),
        network.limits.max_redirects,
    )?;

    Ok(ReqwestClient::<dyn ProgressService>::new(
        network.progress_service.clone(),
        Arc::new(client),
        network.semaphore.clone(),
    ))
}

/// A client from [`vetted_download_client`].
pub(crate) type VettedDownloadClient = ReqwestClient<dyn ProgressService>;

/// [`download_vetted`] through a client built for `allowed_hosts`. The URL is checked against
/// them here as well: the client vets every hop, but the first request's refusal reads better.
pub(crate) async fn download_with(
    request_client: &VettedDownloadClient,
    allowed_hosts: &[String],
    url: &str,
    hash: Option<FileHash>,
    destination: &Path,
    max_bytes: u64,
    on_bytes: Option<BytesProgress<'_>>,
) -> Result<u64, PluginDownloadError> {
    let (url, _) = check_target(url, allowed_hosts)?;

    let mut request = Request::get(url.as_str());
    if let Some(hash) = hash {
        request = request.with_hash(hash);
    }

    request_client
        .download_to_file(request, destination, max_bytes, on_bytes)
        .await
        .map_err(|err| match err {
            // A refused hop surfaces as the same error as a refused first request.
            RequestError::Middleware(err) => match policy_error(err) {
                Ok(policy) => PluginDownloadError::Http(policy),
                Err(err) => PluginDownloadError::Request(RequestError::Middleware(err)),
            },
            err => PluginDownloadError::Request(err),
        })
}

/// Handle `download_to_cache` — download a hash-verified file into the plugin's `/cache`.
pub(crate) async fn handle_download_to_cache(
    plugin_id: &str,
    params: DownloadToCacheParamsDto,
    context: &DownloadContext<'_>,
) -> crate::Result<CachedFileDto> {
    log::debug!(target: "plugin", "[{plugin_id}]: download_to_cache {}", params.url);

    perform_download(params, context, is_blocked_ip)
        .await
        .map_err(|err| {
            log::warn!(target: "plugin", "[{plugin_id}]: download_to_cache failed: {err}");
            crate::ErrorKind::from(err).as_error()
        })
}

// ── Extism host function wrappers ──

host_fn!(
pub download_to_cache(user_data: PluginContext; params: Msgpack<DownloadToCacheParamsDto>) -> MsgpackResult<CachedFileDto> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.clone();
    let hosts = ctx.hosts.clone();
    let limits = ctx.limits;
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    let cache_dir = container.location_info().plugin_cache_dir(&id);

    to_extism_res::<CachedFileDto>(block_on(async {
        let allowed_hosts = hosts.allowed_hosts(&id).await?;
        let download = DownloadContext {
            cache_dir: &cache_dir,
            allowed_hosts: &allowed_hosts,
            semaphore: container.fetch_semaphore(),
            progress_service: container.progress_service(),
            proxy: ProxySource::System,
            limits,
            budget: DOWNLOAD_BUDGET,
        };

        handle_download_to_cache(&id, params.0, &download).await
    }))
});

#[cfg(test)]
mod tests;
