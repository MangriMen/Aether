use std::sync::Arc;

use crate::features::plugins::{
    GitHubPluginFetcher, PluginError, PluginExtractor, PluginSource, PluginSourceStorage,
    PluginStorage,
};

pub struct InstallFromGithubUseCase<
    PE: PluginExtractor,
    PS: PluginStorage,
    Src: PluginSourceStorage,
> {
    plugin_extractor: Arc<PE>,
    plugin_storage: Arc<PS>,
    plugin_source_storage: Arc<Src>,
    github_fetcher: Arc<GitHubPluginFetcher>,
}

impl<PE: PluginExtractor, PS: PluginStorage, Src: PluginSourceStorage>
    InstallFromGithubUseCase<PE, PS, Src>
{
    pub fn new(
        plugin_extractor: Arc<PE>,
        plugin_storage: Arc<PS>,
        plugin_source_storage: Arc<Src>,
        github_fetcher: Arc<GitHubPluginFetcher>,
    ) -> Self {
        Self {
            plugin_extractor,
            plugin_storage,
            plugin_source_storage,
            github_fetcher,
        }
    }

    /// Install a plugin from a GitHub release tag.
    /// Does NOT sync the plugin registry — caller should call `sync()` after.
    pub async fn execute(&self, owner: &str, repo: &str, tag: &str) -> Result<String, PluginError> {
        let releases = self.github_fetcher.fetch_releases(owner, repo, 10).await?;

        let release = releases
            .into_iter()
            .find(|r| r.tag_name == tag)
            .ok_or_else(|| PluginError::GitHubNoAssets {
                owner: owner.to_string(),
                repo: repo.to_string(),
                tag: tag.to_string(),
            })?;

        let zip_bytes = self
            .github_fetcher
            .download_asset(&release.zip_download_url)
            .await?;

        let temp_file = write_bytes_to_temp_file(&zip_bytes)?;
        let extracted = self.plugin_extractor.extract(temp_file.path()).await?;
        let plugin_id = extracted.plugin_id.clone();
        self.plugin_storage.add(extracted).await?;

        let source = PluginSource::GitHub {
            owner: owner.to_string(),
            repo: repo.to_string(),
            current_tag: release.tag_name.clone(),
            current_version: release.version.clone(),
        };

        self.plugin_source_storage.save(&plugin_id, &source).await?;

        Ok(plugin_id)
    }
}

pub fn write_bytes_to_temp_file(bytes: &[u8]) -> Result<tempfile::NamedTempFile, PluginError> {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new()
        .map_err(|e| PluginError::Storage(crate::shared::io::domain::IoError::IoError(e)))?;

    tmp.write_all(bytes)
        .map_err(|e| PluginError::Storage(crate::shared::io::domain::IoError::IoError(e)))?;

    // Flush and rewind so the reader sees all data
    tmp.flush()
        .map_err(|e| PluginError::Storage(crate::shared::io::domain::IoError::IoError(e)))?;

    Ok(tmp)
}
