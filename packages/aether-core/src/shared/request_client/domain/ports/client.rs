use std::path::Path;

use async_trait::async_trait;
use bytes::Bytes;

use crate::{
    features::events::ProgressBarId,
    shared::request_client::{Request, RequestError},
};

/// Called with the number of bytes that arrived since the previous call, as the body streams.
/// Synchronous and called from the transfer itself, so it must not block.
pub type BytesProgress<'a> = &'a (dyn Fn(u64) + Send + Sync);

#[async_trait]
pub trait RequestClient: Send + Sync {
    async fn fetch_bytes(&self, request: Request) -> Result<Bytes, RequestError>;

    async fn fetch_bytes_with_progress(
        &self,
        request: Request,
        loading_bar: Option<(&ProgressBarId, f64)>,
    ) -> Result<Bytes, RequestError>;

    /// Stream the response body to `destination` and return its size in bytes.
    ///
    /// Unlike [`Self::fetch_bytes`], the body is never held in memory, and a non-2xx status is
    /// an error rather than a body. The file only appears under `destination` once it is
    /// complete and, if the request carries a hash, verified: on any failure — status, size,
    /// hash, I/O — nothing is left behind, neither under the final name nor as a partial file.
    /// An existing file at `destination` is replaced.
    ///
    /// `max_size` caps the body; going over it fails the download. `on_bytes` sees every chunk
    /// as it is written; the counts add up to the returned size on success.
    async fn download_to_file(
        &self,
        request: Request,
        destination: &Path,
        max_size: u64,
        on_bytes: Option<BytesProgress<'_>>,
    ) -> Result<u64, RequestError>;
}
