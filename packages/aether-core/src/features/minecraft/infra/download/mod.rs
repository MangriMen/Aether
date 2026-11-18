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

use crate::shared::{
    hash::{FileHash, HashAlgorithm},
    io::domain::IoError,
    request_client::Request,
};

/// A GET of `url`, checked against `sha1` as the version's metadata gives it: a corrupt or
/// truncated download is refused instead of being kept as the file. An empty digest (modded
/// versions' libraries often carry one) leaves the request unchecked; one that is no sha1 is
/// an error, not a reason to skip the check.
fn checked_get(url: &str, sha1: &str) -> Result<Request, IoError> {
    let request = Request::get(url);
    if sha1.is_empty() {
        return Ok(request);
    }
    let hash = FileHash::new(HashAlgorithm::Sha1, sha1).map_err(|err| {
        IoError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("The digest of {url} is not a sha1: {err}"),
        ))
    })?;
    Ok(request.with_hash(hash))
}

#[cfg(test)]
mod checked_get_tests {
    use super::checked_get;

    #[test]
    fn should_leave_the_request_unchecked_when_the_digest_is_empty() {
        assert!(checked_get("https://example.com/a.jar", "").is_ok());
    }

    #[test]
    fn should_accept_a_sha1_digest() {
        assert!(
            checked_get(
                "https://example.com/a.jar",
                "da39a3ee5e6b4b0d3255bfef95601890afd80709"
            )
            .is_ok()
        );
    }

    #[test]
    fn should_refuse_a_digest_that_is_no_sha1() {
        for digest in [
            "not-hex-at-all",
            "abc123",
            "zz39a3ee5e6b4b0d3255bfef95601890afd80709",
        ] {
            assert!(
                checked_get("https://example.com/a.jar", digest).is_err(),
                "{digest}"
            );
        }
    }
}
