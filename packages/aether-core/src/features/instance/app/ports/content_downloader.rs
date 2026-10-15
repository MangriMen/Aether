use std::path::Path;

use async_trait::async_trait;

use crate::{
    features::instance::domain::{InstanceError, ProviderId},
    shared::{hash::FileHash, request_client::BytesProgress},
};

/// Downloads the files of an install plan, each under the network policy of whoever supplied
/// its URL: the provider that returned the plan for a [`ContentSource::Direct`], the one that
/// resolved the ref for a [`ContentSource::ProviderRef`].
///
/// Guards against a plugin reaching hosts outside its `allowed_hosts` through a plan: only a
/// built-in provider's URL goes out through the application's client. Decided by `origin`,
/// never by looking at the host.
///
/// [`ContentSource::Direct`]: crate::features::instance::ContentSource::Direct
/// [`ContentSource::ProviderRef`]: crate::features::instance::ContentSource::ProviderRef
#[async_trait]
pub trait ContentDownloader: Send + Sync {
    /// Download `url` to `destination` and return its size in bytes. With a `hash`, the body
    /// is verified against it while it streams.
    ///
    /// Nothing is left at `destination` on any failure. `on_bytes` is told of each chunk as it
    /// lands.
    async fn download(
        &self,
        origin: &ProviderId,
        url: &str,
        hash: Option<&FileHash>,
        destination: &Path,
        on_bytes: Option<BytesProgress<'_>>,
    ) -> Result<u64, InstanceError>;
}
