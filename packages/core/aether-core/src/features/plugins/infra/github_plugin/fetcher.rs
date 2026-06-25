use std::sync::Arc;

use reqwest_middleware::ClientWithMiddleware;

use crate::features::plugins::{
    GitHubPluginPreview, GitHubReleaseInfo, PluginError, PluginManifestPreview, PluginUpdateInfo,
};

/// Fetches plugin release information from GitHub repositories.
pub struct GitHubPluginFetcher {
    client: Arc<ClientWithMiddleware>,
}

impl GitHubPluginFetcher {
    pub fn new(client: Arc<ClientWithMiddleware>) -> Self {
        Self { client }
    }

    /// Fetch all releases for a GitHub repo, limited to `max_results`.
    #[allow(clippy::items_after_statements)]
    pub async fn fetch_releases(
        &self,
        owner: &str,
        repo: &str,
        max_results: u32,
    ) -> Result<Vec<GitHubReleaseInfo>, PluginError> {
        let url =
            format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page={max_results}");

        let response = self
            .client
            .get(&url)
            .header("User-Agent", "Aether")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| PluginError::GitHubFetchError {
                owner: owner.to_string(),
                repo: repo.to_string(),
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
            return Err(PluginError::GitHubFetchError {
                owner: owner.to_string(),
                repo: repo.to_string(),
                details: format!("HTTP {status}"),
            });
        }

        let releases: Vec<GitHubRelease> =
            response
                .json()
                .await
                .map_err(|e| PluginError::GitHubFetchError {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
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
    ) -> Result<PluginUpdateInfo, PluginError> {
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

    /// Download a file from a URL and return the bytes.
    pub async fn download_asset(&self, url: &str) -> Result<Vec<u8>, PluginError> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", "Aether")
            .send()
            .await
            .map_err(|e| PluginError::DownloadFailed {
                url: url.to_string(),
                details: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(PluginError::DownloadFailed {
                url: url.to_string(),
                details: format!("HTTP {}", response.status()),
            });
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| PluginError::DownloadFailed {
                url: url.to_string(),
                details: e.to_string(),
            })
    }

    /// Preview a plugin from a GitHub repo URL.
    /// Fetches releases, manifest and capabilities from the latest release.
    pub async fn preview_plugin(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubPluginPreview, PluginError> {
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
                match self.download_asset(manifest_url).await {
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
                self.download_asset(cap_url)
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
