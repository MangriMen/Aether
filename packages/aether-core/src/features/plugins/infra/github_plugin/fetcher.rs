use std::sync::Arc;

use reqwest_middleware::ClientWithMiddleware;

use super::types::{GitHubPluginPreview, GitHubReleaseInfo, PluginUpdateInfo};
use crate::features::plugins::{PluginManifestPreview, PluginProviderError, PluginSourceType};

/// Upper bound for a downloaded plugin archive. The body is buffered in memory before it is
/// written to a temporary file, so this bounds the process's memory rather than the disk.
const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;

/// Upper bound for a manifest or capabilities file. Both are small JSON documents; a release
/// asset that claims to be one has no reason to be large.
const MAX_METADATA_BYTES: u64 = 1024 * 1024;

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

/// Reads the body chunk by chunk so an oversized asset is rejected before it fills memory.
async fn read_capped(
    mut response: reqwest::Response,
    url: &str,
    limit: u64,
) -> Result<Vec<u8>, PluginProviderError> {
    let too_large = || PluginProviderError::DownloadFailed {
        url: url.to_string(),
        details: format!("asset exceeds {limit} bytes"),
    };
    if response.content_length().is_some_and(|len| len > limit) {
        return Err(too_large());
    }
    let mut body = Vec::new();
    while let Some(chunk) =
        response
            .chunk()
            .await
            .map_err(|e| PluginProviderError::DownloadFailed {
                url: url.to_string(),
                details: e.to_string(),
            })?
    {
        if body.len() as u64 + chunk.len() as u64 > limit {
            return Err(too_large());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
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
                // Identify assets by file extension pattern
                let manifest_asset = r.assets.iter().find(|a| a.name.ends_with(".manifest.json"));
                let capabilities_asset = r
                    .assets
                    .iter()
                    .find(|a| a.name.ends_with(".capabilities.json"));
                let zip_asset = r.assets.iter().find(|a| {
                    std::path::Path::new(&a.name)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
                });

                // Derive plugin_id from manifest filename if present
                let plugin_id = manifest_asset
                    .and_then(|a| a.name.strip_suffix(".manifest.json"))
                    .map(String::from);

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
                    manifest_download_url: manifest_asset.map(|a| a.browser_download_url.clone()),
                    capabilities_download_url: capabilities_asset
                        .map(|a| a.browser_download_url.clone()),
                    plugin_id,
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

    /// Download a plugin archive from a URL and return the bytes.
    pub async fn download_asset(&self, url: &str) -> Result<Vec<u8>, PluginProviderError> {
        self.download_asset_capped(url, MAX_ARCHIVE_BYTES).await
    }

    /// Download a file from a URL and return the bytes, refusing a body over `limit`.
    async fn download_asset_capped(
        &self,
        url: &str,
        limit: u64,
    ) -> Result<Vec<u8>, PluginProviderError> {
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

        read_capped(response, url, limit).await
    }

    /// Preview a plugin from a GitHub repo URL.
    /// Fetches releases, manifest and capabilities from the latest release.
    pub async fn preview_plugin(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubPluginPreview, PluginProviderError> {
        let mut releases = self.fetch_releases(owner, repo, 20).await?;

        // Try to get manifest + capabilities from the first release that has a manifest
        let (manifest, capabilities) = self.fetch_manifest_and_capabilities(&releases).await;

        // Propagate plugin_id to all releases if we found it
        if let Some(ref manifest) = manifest {
            for release in &mut releases {
                if release.plugin_id.is_none() {
                    release.plugin_id = Some(manifest.id.clone());
                }
            }
        }

        Ok(GitHubPluginPreview {
            owner: owner.to_string(),
            repo: repo.to_string(),
            manifest,
            capabilities,
            releases,
        })
    }

    /// Try to fetch manifest and capabilities from releases' assets.
    async fn fetch_manifest_and_capabilities(
        &self,
        releases: &[GitHubReleaseInfo],
    ) -> (Option<PluginManifestPreview>, Option<String>) {
        for release in releases {
            // Fetch manifest
            let manifest = if let Some(manifest_url) = &release.manifest_download_url {
                match self
                    .download_asset_capped(manifest_url, MAX_METADATA_BYTES)
                    .await
                {
                    Ok(bytes) => serde_json::from_slice::<
                        aether_core_plugin_api::v0::PluginManifestDto,
                    >(&bytes)
                    .ok()
                    .map(|dto| PluginManifestPreview {
                        id: dto.metadata.id,
                        name: dto.metadata.name,
                        version: dto.metadata.version,
                        description: dto.metadata.description,
                        authors: dto.metadata.authors,
                        license: dto.metadata.license,
                        api_version: Some(dto.api.version),
                    }),
                    Err(_) => continue,
                }
            } else {
                continue;
            };

            // Fetch capabilities if URL is present
            let capabilities = if let Some(cap_url) = &release.capabilities_download_url {
                self.download_asset_capped(cap_url, MAX_METADATA_BYTES)
                    .await
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
            } else {
                None
            };

            if manifest.is_some() {
                return (manifest, capabilities);
            }
        }
        (None, None)
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

    #[tokio::test]
    async fn should_read_a_body_within_the_limit() {
        let body = read_capped(response(b"hello"), "u", 5).await.unwrap();
        assert_eq!(body, b"hello");
    }

    #[tokio::test]
    async fn should_reject_a_body_over_the_limit() {
        let err = read_capped(response(b"hello!"), "u", 5).await.unwrap_err();
        assert!(matches!(err, PluginProviderError::DownloadFailed { .. }));
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
