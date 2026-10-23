mod assets_service;
mod client_service;
mod libraries_service;
mod logging_service;
mod minecraft_download_cache_keys;
mod minecraft_download_resolver;
mod minecraft_download_service;
#[cfg(test)]
mod tests;
mod version_progress;

pub use assets_service::AssetsService;
pub use client_service::ClientService;
pub use libraries_service::LibrariesService;
pub use logging_service::LoggingService;
pub(crate) use minecraft_download_cache_keys::MinecraftDownloadCacheNamespaces;
pub(crate) use minecraft_download_cache_keys::assets_index_key;
pub(crate) use minecraft_download_cache_keys::version_info_key;
pub(crate) use minecraft_download_cache_keys::version_jar_key;
pub use minecraft_download_resolver::MinecraftDownloadResolver;
pub use minecraft_download_service::MinecraftDownloadService;
use version_progress::VersionProgress;

use crate::shared::{hash::FileHash, request_client::Request};

/// A GET of `url`, checked against `sha1` as the version's metadata gives it: a corrupt or
/// truncated download is refused instead of being kept as the file. A digest that is none
/// (modded versions' libraries often carry an empty one) leaves the request unchecked.
fn checked_get(url: &str, sha1: &str) -> Request {
    let request = Request::get(url);
    match FileHash::parse("sha1", sha1) {
        Ok(hash) => request.with_hash(hash),
        Err(_) => request,
    }
}
