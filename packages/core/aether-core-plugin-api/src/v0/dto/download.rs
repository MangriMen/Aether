use serde::{Deserialize, Serialize};

use super::FileHashDto;

/// Parameters of the `download_to_cache` host function.
///
/// The host downloads `url` into the plugin's `/cache`, verifies it against `hash` and only
/// then makes it visible. The network policy is the one of `http_get` (host `allowed_hosts`,
/// public addresses only, at most 5 redirects, each hop checked like the first request).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadToCacheParamsDto {
    /// Absolute URL. Only the `http` and `https` schemes are accepted.
    pub url: String,
    /// Expected digest of the file. Required: it is both the integrity check and the file's
    /// name in the cache.
    pub hash: FileHashDto,
}

/// A verified file in the plugin's cache, returned by `download_to_cache`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CachedFileDto {
    /// Plugin-side path, readable through WASI: `/cache/downloads/<algorithm>/<hex digest>`.
    /// The name is derived from the hash by the host, never from the plugin.
    pub path: String,
    /// File size in bytes.
    pub size: u64,
}
