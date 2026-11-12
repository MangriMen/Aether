use std::sync::Arc;

use reqwest_middleware::ClientWithMiddleware;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::shared::io::domain::IoError;

use super::types::{GitHubPluginPreview, GitHubReleaseInfo, PluginUpdateInfo};
use crate::features::plugins::{PluginProviderError, PluginSourceType};

/// Upper bound for a downloaded plugin archive. The body goes to disk as it arrives, so this stops
/// an endless stream rather than limiting what a plugin may weigh.
const MAX_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;

/// Only GitHub itself may serve plugin assets; the URL can come from a user-pasted repo.
fn is_allowed_asset_url(url: &str) -> bool {
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return false;
    };
    if parsed.scheme() != "https" {
        return false;
    }
    parsed.host_str().is_some_and(|host| {
        host == "github.com" || host == "api.github.com" || host.ends_with(".githubusercontent.com")
    })
}

/// Copies the body to `dest` chunk by chunk, so an oversized asset is rejected as soon as it
/// crosses `limit` instead of after it has been taken in whole. What was written before a failure
/// stays in `dest`: whoever owns it throws it away.
async fn copy_capped<W: AsyncWrite + Unpin>(
    mut response: reqwest::Response,
    url: &str,
    limit: u64,
    dest: &mut W,
) -> Result<u64, PluginProviderError> {
    let failed = |details: String| PluginProviderError::DownloadFailed {
        url: url.to_string(),
        details,
    };
    let too_large = || failed(format!("asset exceeds {limit} bytes"));
    if response.content_length().is_some_and(|len| len > limit) {
        return Err(too_large());
    }
    let mut written = 0u64;
    while let Some(chunk) = response.chunk().await.map_err(|e| failed(e.to_string()))? {
        written += chunk.len() as u64;
        if written > limit {
            return Err(too_large());
        }
        // A full disk is not a failure of the download: it is told apart from one.
        dest.write_all(&chunk).await.map_err(IoError::from)?;
    }
    dest.flush().await.map_err(IoError::from)?;
    Ok(written)
}

/// Fetches plugin release information from GitHub repositories.
pub struct GitHubPluginFetcher {
    client: Arc<ClientWithMiddleware>,
}

impl GitHubPluginFetcher {
    pub fn new(client: Arc<ClientWithMiddleware>) -> Self {
        Self { client }
    }

    /// Check if the response status indicates rate limiting.
    fn check_rate_limit(
        status: reqwest::StatusCode,
        _headers: &reqwest::header::HeaderMap,
    ) -> bool {
        status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS
    }

    /// Extract retry-after seconds from response headers.
    fn retry_after_from(headers: &reqwest::header::HeaderMap) -> Option<u32> {
        headers
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
    }

    /// Build a rate-limit error when applicable.
    fn rate_limit_error_or(
        status: reqwest::StatusCode,
        headers: &reqwest::header::HeaderMap,
        fallback: PluginProviderError,
    ) -> PluginProviderError {
        if Self::check_rate_limit(status, headers) {
            return PluginProviderError::ProviderRateLimited {
                source_type: PluginSourceType::GitHub,
                retry_after: Self::retry_after_from(headers),
            };
        }
        fallback
    }

    /// Fetch all releases for a GitHub repo, limited to `max_results`.
    #[allow(clippy::items_after_statements)]
    pub async fn fetch_releases(
        &self,
        owner: &str,
        repo: &str,
        max_results: u32,
    ) -> Result<Vec<GitHubReleaseInfo>, PluginProviderError> {
        let url =
            format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page={max_results}");

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "Aether")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| PluginProviderError::ProviderFetchError {
                source_type: PluginSourceType::GitHub,
                details: e.to_string(),
            })?;

        #[derive(serde::Deserialize)]
        struct GitHubRelease {
            tag_name: String,
            prerelease: bool,
            published_at: String,
            html_url: String,
            assets: Vec<GitHubAsset>,
        }

        #[derive(serde::Deserialize)]
        struct GitHubAsset {
            name: String,
            browser_download_url: String,
            size: u64,
        }

        if !response.status().is_success() {
            let status = response.status();
            let headers = response.headers().clone();
            return Err(Self::rate_limit_error_or(
                status,
                &headers,
                PluginProviderError::ProviderFetchError {
                    source_type: PluginSourceType::GitHub,
                    details: format!("HTTP {status}"),
                },
            ));
        }

        let releases: Vec<GitHubRelease> =
            response
                .json()
                .await
                .map_err(|e| PluginProviderError::ProviderFetchError {
                    source_type: PluginSourceType::GitHub,
                    details: format!("Failed to parse response: {e}"),
                })?;

        Ok(releases
            .into_iter()
            .map(|r| {
                // A release is installable when it has exactly one `.zip` asset. With none there
                // is nothing to install; with several there is no telling which one is the plugin,
                // and guessing would install a file the author may not have meant, so such a
                // release is listed without a download address, like one with no archive. An
                // archive whose size, as GitHub reports it, is over the limit is left out the
                // same way, before any byte of it is requested.
                let mut zips = r.assets.iter().filter(|a| {
                    std::path::Path::new(&a.name)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
                });
                let zip_asset = match (zips.next(), zips.next()) {
                    (Some(only), None) if only.size <= MAX_ARCHIVE_BYTES => Some(only),
                    _ => None,
                };

                let version = r.tag_name.trim_start_matches('v').to_string();

                GitHubReleaseInfo {
                    tag_name: r.tag_name,
                    version,
                    is_prerelease: r.prerelease,
                    published_at: r.published_at,
                    html_url: r.html_url,
                    zip_download_url: zip_asset
                        .map(|a| a.browser_download_url.clone())
                        .unwrap_or_default(),
                }
            })
            .collect())
    }

    /// Check if a plugin has updates available.
    pub async fn check_for_updates(
        &self,
        owner: &str,
        repo: &str,
        current_tag: &str,
        current_version: &str,
        max_releases: u32,
    ) -> Result<PluginUpdateInfo, PluginProviderError> {
        let all_releases = self.fetch_releases(owner, repo, max_releases).await?;

        // Find the latest non-prerelease version, falling back to any release
        let latest = all_releases
            .iter()
            .find(|r| !r.is_prerelease)
            .or_else(|| all_releases.first());

        let (latest_version, latest_tag, has_update) = match latest {
            Some(latest) if latest.tag_name != current_tag => {
                // Check if latest is actually newer by semver
                let has_update = match (
                    semver::Version::parse(&latest.version),
                    semver::Version::parse(current_version),
                ) {
                    (Ok(latest_ver), Ok(current_ver)) => latest_ver > current_ver,
                    _ => true,
                };
                (
                    Some(latest.version.clone()),
                    Some(latest.tag_name.clone()),
                    has_update,
                )
            }
            _ => (None, None, false),
        };

        Ok(PluginUpdateInfo {
            current_version: current_version.to_string(),
            current_tag: current_tag.to_string(),
            latest_version,
            latest_tag,
            has_update,
            all_releases,
        })
    }

    /// Download a plugin archive from a URL into `dest`, returning the number of bytes written.
    /// On failure `dest` may hold a part of the archive.
    pub async fn download_asset<W: AsyncWrite + Unpin>(
        &self,
        url: &str,
        dest: &mut W,
    ) -> Result<u64, PluginProviderError> {
        let response = self.asset_response(url).await?;
        copy_capped(response, url, MAX_ARCHIVE_BYTES, dest).await
    }

    /// Asks GitHub for an asset and checks the answer; the body is left to the caller.
    async fn asset_response(&self, url: &str) -> Result<reqwest::Response, PluginProviderError> {
        if !is_allowed_asset_url(url) {
            return Err(PluginProviderError::DownloadFailed {
                url: url.to_string(),
                details: "asset URL is not served by GitHub over https".to_string(),
            });
        }
        let response = self
            .client
            .get(url)
            .header("User-Agent", "Aether")
            .send()
            .await
            .map_err(|e| PluginProviderError::DownloadFailed {
                url: url.to_string(),
                details: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let headers = response.headers().clone();
            return Err(Self::rate_limit_error_or(
                status,
                &headers,
                PluginProviderError::DownloadFailed {
                    url: url.to_string(),
                    details: format!("HTTP {status}"),
                },
            ));
        }

        Ok(response)
    }

    /// Preview a plugin from a GitHub repo URL: the releases only. What a release holds is read
    /// from its archive, which is not downloaded here.
    pub async fn preview_plugin(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubPluginPreview, PluginProviderError> {
        let releases = self.fetch_releases(owner, repo, 20).await?;

        Ok(GitHubPluginPreview {
            owner: owner.to_string(),
            repo: repo.to_string(),
            releases,
        })
    }

    /// Parse a GitHub URL to extract owner and repo.
    /// Supports: <https://github.com/owner/repo>, <https://github.com/owner/repo/releases/tag/v1.0.0>
    pub fn parse_github_url(url: &str) -> Option<(String, String)> {
        let url = url.trim().trim_end_matches('/');

        // Remove .git suffix if present
        let url = url.strip_suffix(".git").unwrap_or(url);

        // Extract from various URL formats
        let parts: Vec<&str> = url.split("github.com/").collect();
        let path = parts.get(1)?;

        let segments: Vec<&str> = path.split('/').collect();

        match segments.len() {
            // github.com/owner/repo
            2 => Some((segments[0].to_string(), segments[1].to_string())),
            // github.com/owner/repo/releases/...  or  github.com/owner/repo/tree/...
            n if n >= 2 => Some((segments[0].to_string(), segments[1].to_string())),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(body: &'static [u8]) -> reqwest::Response {
        reqwest::Response::from(http::Response::new(body))
    }

    /// A body in `chunks` with no `Content-Length`, the way a chunked answer arrives; an `Err`
    /// among them is a connection that dropped.
    fn chunked(chunks: Vec<Result<&'static [u8], std::io::Error>>) -> reqwest::Response {
        let body = reqwest::Body::wrap_stream(futures::stream::iter(chunks));
        reqwest::Response::from(http::Response::new(body))
    }

    #[tokio::test]
    async fn should_write_the_body_to_the_destination_when_it_is_within_the_limit() {
        let mut dest = Vec::new();

        let written = copy_capped(response(b"hello"), "u", 5, &mut dest)
            .await
            .unwrap();

        assert_eq!((written, dest.as_slice()), (5, &b"hello"[..]));
    }

    #[tokio::test]
    async fn should_write_nothing_when_the_declared_length_is_over_the_limit() {
        let mut dest = Vec::new();

        let err = copy_capped(response(b"hello!"), "u", 5, &mut dest)
            .await
            .unwrap_err();

        assert!(matches!(err, PluginProviderError::DownloadFailed { .. }));
        assert!(dest.is_empty());
    }

    #[tokio::test]
    async fn should_stop_a_chunked_body_when_it_crosses_the_limit() {
        let mut dest = Vec::new();

        let err = copy_capped(
            chunked(vec![Ok(b"abc"), Ok(b"def"), Ok(b"ghi")]),
            "u",
            5,
            &mut dest,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, PluginProviderError::DownloadFailed { .. }));
        // The chunk that crossed the limit was not written.
        assert_eq!(dest, b"abc");
    }

    #[tokio::test]
    async fn should_fail_when_the_connection_drops_in_the_middle_of_the_body() {
        let mut dest = Vec::new();

        let err = copy_capped(
            chunked(vec![
                Ok(b"abc"),
                Err(std::io::Error::other("connection reset")),
            ]),
            "u",
            100,
            &mut dest,
        )
        .await
        .unwrap_err();

        assert!(matches!(err, PluginProviderError::DownloadFailed { .. }));
    }

    #[tokio::test]
    async fn should_report_a_storage_error_when_the_destination_cannot_be_written() {
        let (mut dest, reader) = tokio::io::duplex(1);
        drop(reader);

        let err = copy_capped(response(b"hello"), "u", 100, &mut dest)
            .await
            .unwrap_err();

        assert!(matches!(err, PluginProviderError::Storage(_)));
    }

    #[test]
    fn should_only_allow_github_https_asset_urls() {
        for ok in [
            "https://github.com/o/r/releases/download/v1/a.zip",
            "https://api.github.com/repos/o/r/releases/assets/1",
            "https://objects.githubusercontent.com/x",
        ] {
            assert!(is_allowed_asset_url(ok), "{ok}");
        }
        for bad in [
            "http://github.com/a",
            "https://evil.example/a",
            "https://github.com.evil.example/a",
            "https://evilgithubusercontent.com/a",
            "file:///etc/passwd",
            "not a url",
        ] {
            assert!(!is_allowed_asset_url(bad), "{bad}");
        }
    }
}
