use std::sync::Arc;

use crate::features::plugins::{
    PluginError, PluginExtractor, PluginSource, PluginSourceStorage, PluginStorage,
};

use super::{PluginProviderFactory, install_from_github::write_bytes_to_temp_file};

/// Updates a plugin to a specific version (or latest) using the provider factory.
/// Works with any provider type (GitHub, Modrinth, etc.).
pub struct UpdatePluginUseCase<PE: PluginExtractor, PS: PluginStorage, Src: PluginSourceStorage> {
    plugin_extractor: Arc<PE>,
    plugin_storage: Arc<PS>,
    plugin_source_storage: Arc<Src>,
    provider_factory: Arc<PluginProviderFactory>,
}

impl<PE: PluginExtractor, PS: PluginStorage, Src: PluginSourceStorage>
    UpdatePluginUseCase<PE, PS, Src>
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        plugin_extractor: Arc<PE>,
        plugin_storage: Arc<PS>,
        plugin_source_storage: Arc<Src>,
        provider_factory: Arc<PluginProviderFactory>,
    ) -> Self {
        Self {
            plugin_extractor,
            plugin_storage,
            plugin_source_storage,
            provider_factory,
        }
    }

    pub async fn execute(
        &self,
        plugin_id: &str,
        target_tag: Option<&str>,
    ) -> Result<(), PluginError> {
        let source = self
            .plugin_source_storage
            .get(plugin_id)
            .await?
            .ok_or_else(|| PluginError::NotAGitHubPlugin {
                plugin_id: plugin_id.to_string(),
            })?;

        let source_type = source.to_source_type();
        let identifier = source_identifier(&source);

        let provider = self
            .provider_factory
            .get_provider(&source_type)
            .ok_or_else(|| PluginError::NotAGitHubPlugin {
                plugin_id: plugin_id.to_string(),
            })?;

        let tag = if let Some(t) = target_tag {
            t.to_string()
        } else {
            let current_tag = source.current_tag().unwrap_or_default();
            let updates = provider
                .fetch_latest_version(&identifier, current_tag, "")
                .await?;

            updates
                .latest_tag
                .ok_or_else(|| PluginError::GitHubFetchError {
                    owner: identifier.clone(),
                    repo: identifier.clone(),
                    details: "No releases found".to_string(),
                })?
        };

        // Fetch preview to get all releases with download URLs
        let preview = provider.fetch_preview(&identifier).await?;

        let release = preview
            .releases
            .into_iter()
            .find(|r| r.tag_name == tag)
            .ok_or_else(|| PluginError::GitHubNoAssets {
                owner: identifier.clone(),
                repo: identifier.clone(),
                tag: tag.clone(),
            })?;

        // Download zip to temp file, reuse PluginExtractor
        let zip_bytes = provider.download_plugin(&release.download_url).await?;
        let temp_file = write_bytes_to_temp_file(&zip_bytes)?;

        // Remove old plugin files & install the new version
        self.plugin_storage.remove(plugin_id).await?;
        let extracted = self.plugin_extractor.extract(temp_file.path()).await?;
        self.plugin_storage.add(extracted).await?;

        // Update source info
        let (owner_str, repo_str) = identifier.rsplit_once('/').unwrap_or((&identifier, ""));
        let updated_source = PluginSource::GitHub {
            owner: owner_str.to_string(),
            repo: repo_str.to_string(),
            current_tag: release.tag_name.clone(),
            current_version: release.version.clone(),
        };
        self.plugin_source_storage
            .save(plugin_id, &updated_source)
            .await?;

        Ok(())
    }
}

/// Build a provider identifier from a `PluginSource`.
fn source_identifier(source: &PluginSource) -> String {
    match source {
        PluginSource::GitHub { owner, repo, .. } => format!("{owner}/{repo}"),
        PluginSource::Local => String::new(),
    }
}
