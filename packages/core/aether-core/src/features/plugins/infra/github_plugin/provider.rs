use std::sync::Arc;

use async_trait::async_trait;
use reqwest_middleware::ClientWithMiddleware;

use super::fetcher::GitHubPluginFetcher;
use crate::features::plugins::app::PluginProvider;
use crate::features::plugins::domain::{
    PluginError, PluginSourceType, ProviderPluginPreview, ProviderReleaseInfo, ProviderUpdateInfo,
};

/// Concrete provider for GitHub-sourced plugins.
///
/// Wraps `GitHubPluginFetcher` and implements the `PluginProvider` trait,
/// translating GitHub-specific types into generic provider types.
pub struct GithubProvider {
    fetcher: GitHubPluginFetcher,
}

impl GithubProvider {
    pub fn new(client: Arc<ClientWithMiddleware>) -> Self {
        Self {
            fetcher: GitHubPluginFetcher::new(client),
        }
    }
}

#[async_trait]
impl PluginProvider for GithubProvider {
    fn source_type(&self) -> PluginSourceType {
        PluginSourceType::GitHub
    }

    fn parse_identifier(&self, raw: &str) -> Result<String, PluginError> {
        let (owner, repo) = GitHubPluginFetcher::parse_github_url(raw).ok_or_else(|| {
            PluginError::ProviderFetchError {
                source_type: PluginSourceType::GitHub,
                details: format!("Cannot parse GitHub URL: {raw}"),
            }
        })?;
        Ok(format!("{owner}/{repo}"))
    }

    async fn fetch_preview(&self, identifier: &str) -> Result<ProviderPluginPreview, PluginError> {
        let (owner, repo) = split_identifier(identifier);
        let preview = self.fetcher.preview_plugin(owner, repo).await?;

        Ok(ProviderPluginPreview {
            owner: preview.owner,
            repo: preview.repo,
            manifest: preview.manifest,
            capabilities: preview.capabilities,
            releases: preview
                .releases
                .into_iter()
                .map(|r| ProviderReleaseInfo {
                    tag_name: r.tag_name,
                    version: r.version,
                    is_prerelease: r.is_prerelease,
                    published_at: r.published_at,
                    html_url: r.html_url,
                    download_url: r.zip_download_url,
                })
                .collect(),
        })
    }

    async fn fetch_latest_version(
        &self,
        identifier: &str,
        current_tag: &str,
        current_version: &str,
    ) -> Result<ProviderUpdateInfo, PluginError> {
        let (owner, repo) = split_identifier(identifier);
        let info = self
            .fetcher
            .check_for_updates(owner, repo, current_tag, current_version, 20)
            .await?;

        Ok(ProviderUpdateInfo {
            current_version: info.current_version,
            current_tag: info.current_tag,
            latest_version: info.latest_version,
            latest_tag: info.latest_tag,
            has_update: info.has_update,
            all_releases: info
                .all_releases
                .into_iter()
                .map(|r| ProviderReleaseInfo {
                    tag_name: r.tag_name,
                    version: r.version,
                    is_prerelease: r.is_prerelease,
                    published_at: r.published_at,
                    html_url: r.html_url,
                    download_url: r.zip_download_url,
                })
                .collect(),
        })
    }

    async fn download_plugin(&self, download_url: &str) -> Result<Vec<u8>, PluginError> {
        self.fetcher.download_asset(download_url).await
    }
}

/// Splits an "owner/repo" string into (&owner, &repo).
fn split_identifier(identifier: &str) -> (&str, &str) {
    if let Some((owner, repo)) = identifier.split_once('/') {
        (owner, repo)
    } else {
        (identifier, identifier)
    }
}
