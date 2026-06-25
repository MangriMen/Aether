use std::str::FromStr;
use std::{path::PathBuf, sync::Arc};

use crate::{
    core::LazyLocator,
    features::plugins::{
        DisablePluginUseCase, EditPluginSettings, EditPluginSettingsUseCase, EnablePluginUseCase,
        ForceEnablePluginUseCase, GetPluginApiVersionUseCase, GetPluginDtoUseCase,
        GetPluginSettingsUseCase, ImportPluginsUseCase, ListPluginsDtoUseCase, PluginDto,
        PluginExtractor, PluginSettings, PluginSource, PluginSourceStorage, PluginSourceType,
        PluginStorage, ProviderUpdateInfo, RemovePluginUseCase, SyncPluginsUseCase,
        write_bytes_to_temp_file,
    },
};

#[tracing::instrument]
pub async fn import(paths: Vec<PathBuf>) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    let disable_plugin_use_case = DisablePluginUseCase::new(
        locator.get_plugin_registry().await,
        locator.get_plugin_loader_registry().await,
        locator.get_settings_storage().await,
    );

    let sync_plugins_use_case = Arc::new(SyncPluginsUseCase::new(
        locator.get_plugin_storage().await,
        locator.get_plugin_registry().await,
        disable_plugin_use_case,
        locator.get_event_emitter().await,
    ));

    Ok(ImportPluginsUseCase::new(
        locator.get_plugin_extractor().await,
        locator.get_plugin_storage().await,
        sync_plugins_use_case,
    )
    .execute(paths)
    .await?)
}

pub async fn sync() -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    let disable_plugin_use_case = DisablePluginUseCase::new(
        locator.get_plugin_registry().await,
        locator.get_plugin_loader_registry().await,
        locator.get_settings_storage().await,
    );

    Ok(SyncPluginsUseCase::new(
        locator.get_plugin_storage().await,
        locator.get_plugin_registry().await,
        disable_plugin_use_case,
        locator.get_event_emitter().await,
    )
    .execute()
    .await?)
}

pub async fn list() -> crate::Result<Vec<PluginDto>> {
    let locator = LazyLocator::get().await?;

    Ok(
        ListPluginsDtoUseCase::new(locator.get_plugin_registry().await)
            .execute()
            .await?,
    )
}

pub async fn get(plugin_id: String) -> crate::Result<PluginDto> {
    let locator = LazyLocator::get().await?;

    Ok(
        GetPluginDtoUseCase::new(locator.get_plugin_registry().await)
            .execute(plugin_id)
            .await?,
    )
}

#[tracing::instrument]
pub async fn remove(plugin_id: String) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    let disable_plugin_use_case = DisablePluginUseCase::new(
        locator.get_plugin_registry().await,
        locator.get_plugin_loader_registry().await,
        locator.get_settings_storage().await,
    );

    let sync_plugins_use_case = Arc::new(SyncPluginsUseCase::new(
        locator.get_plugin_storage().await,
        locator.get_plugin_registry().await,
        disable_plugin_use_case,
        locator.get_event_emitter().await,
    ));

    Ok(
        RemovePluginUseCase::new(locator.get_plugin_storage().await, sync_plugins_use_case)
            .execute(plugin_id)
            .await?,
    )
}

#[tracing::instrument]
pub async fn enable(plugin_id: String) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(EnablePluginUseCase::new(
        locator.get_plugin_registry().await,
        locator.get_plugin_loader_registry().await,
        locator.get_plugin_settings_storage().await,
        locator.get_settings_storage().await,
    )
    .execute(plugin_id)
    .await?)
}

/// Force-enable a plugin despite an API version major mismatch.
/// The user acknowledges the risk and the plugin will be loaded anyway.
#[tracing::instrument]
pub async fn force_enable(plugin_id: String) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(ForceEnablePluginUseCase::new(
        locator.get_plugin_registry().await,
        locator.get_plugin_loader_registry().await,
        locator.get_plugin_settings_storage().await,
        locator.get_settings_storage().await,
    )
    .execute(plugin_id)
    .await?)
}

#[tracing::instrument]
pub async fn disable(plugin_id: String) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(DisablePluginUseCase::new(
        locator.get_plugin_registry().await,
        locator.get_plugin_loader_registry().await,
        locator.get_settings_storage().await,
    )
    .execute(plugin_id)
    .await?)
}

#[tracing::instrument]
pub async fn call(plugin_id: String, data: String) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    let plugin_registry = locator.get_plugin_registry().await;
    let _plugin = plugin_registry.get(&plugin_id);

    log::debug!("Calling plugin {:?}", plugin_id);
    // plugin.plugin.call(data).await?;

    Ok(())
}

pub async fn get_settings(plugin_id: String) -> crate::Result<Option<PluginSettings>> {
    let locator = LazyLocator::get().await?;

    Ok(
        GetPluginSettingsUseCase::new(locator.get_plugin_settings_storage().await)
            .execute(plugin_id)
            .await?,
    )
}

#[tracing::instrument]
pub async fn edit_settings(
    plugin_id: String,
    edit_settings: EditPluginSettings,
) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(
        EditPluginSettingsUseCase::new(locator.get_plugin_settings_storage().await)
            .execute(plugin_id, edit_settings)
            .await?,
    )
}

pub async fn get_api_version() -> crate::Result<semver::Version> {
    Ok(GetPluginApiVersionUseCase::default().execute().await?)
}

// ── Provider factory API ──

/// Return all registered plugin provider source types.
pub async fn get_available_providers() -> crate::Result<Vec<PluginSourceType>> {
    let locator = LazyLocator::get().await?;
    let factory = locator.get_plugin_provider_factory().await;
    Ok(factory.list_source_types())
}

// ── GitHub / Remote source operations ──

/// Install a plugin from a GitHub release.
pub async fn install_from_github(
    owner: String,
    repo: String,
    tag: String,
) -> crate::Result<String> {
    let locator = LazyLocator::get().await?;

    let plugin_source_storage = locator.get_plugin_source_storage().await;
    let github_fetcher = locator.get_github_plugin_fetcher().await;

    let plugin_id = crate::features::plugins::InstallFromGithubUseCase::new(
        locator.get_plugin_extractor().await,
        locator.get_plugin_storage().await,
        plugin_source_storage,
        github_fetcher,
    )
    .execute(&owner, &repo, &tag)
    .await?;

    // Sync the plugin registry so the new plugin appears
    sync().await?;

    Ok(plugin_id)
}

/// Get the source info for a plugin (if it was installed from GitHub).
pub async fn get_plugin_source(plugin_id: String) -> crate::Result<Option<PluginSource>> {
    let locator = LazyLocator::get().await?;
    Ok(locator
        .get_plugin_source_storage()
        .await
        .get(&plugin_id)
        .await?)
}

/// Check for plugin updates using the provider factory.
pub async fn check_for_updates(plugin_id: String) -> crate::Result<ProviderUpdateInfo> {
    let locator = LazyLocator::get().await?;

    Ok(crate::features::plugins::CheckForPluginUpdatesUseCase::new(
        locator.get_plugin_source_storage().await,
        locator.get_plugin_provider_factory().await,
    )
    .execute(&plugin_id)
    .await?)
}

/// Update a plugin to a newer version (or a specific tag) using the provider factory.
pub async fn update_plugin(plugin_id: String, target_tag: Option<String>) -> crate::Result<()> {
    let locator = LazyLocator::get().await?;

    Ok(crate::features::plugins::UpdatePluginUseCase::new(
        locator.get_plugin_extractor().await,
        locator.get_plugin_storage().await,
        locator.get_plugin_source_storage().await,
        locator.get_plugin_provider_factory().await,
    )
    .execute(&plugin_id, target_tag.as_deref())
    .await?)
}

/// Preview a plugin from a GitHub URL (e.g. <https://github.com/owner/repo>).
/// Returns releases and manifest info without installing.
pub async fn preview_plugin_from_github(
    url: String,
) -> crate::Result<crate::features::plugins::GitHubPluginPreview> {
    use crate::features::plugins::infra::GitHubPluginFetcher;

    let (owner, repo) = GitHubPluginFetcher::parse_github_url(&url).ok_or_else(|| {
        crate::ErrorKind::CoreError(format!("Invalid GitHub URL: {url}")).as_error()
    })?;

    let locator = LazyLocator::get().await?;
    let github_fetcher = locator.get_github_plugin_fetcher().await;

    github_fetcher
        .preview_plugin(&owner, &repo)
        .await
        .map_err(Into::into)
}

// ── Generic provider-based API ──

/// Preview a plugin using the provider factory.
/// `source_type` is a string like "github".
/// `identifier` is a provider-specific identifier (e.g., GitHub URL or "owner/repo").
pub async fn preview_plugin_from_provider(
    source_type: String,
    identifier: String,
) -> crate::Result<crate::features::plugins::ProviderPluginPreview> {
    let source_type =
        crate::features::plugins::PluginSourceType::from_str(&source_type).map_err(|()| {
            crate::ErrorKind::CoreError(format!("Unknown plugin source type: {source_type}"))
                .as_error()
        })?;

    let locator = LazyLocator::get().await?;
    let factory = locator.get_plugin_provider_factory().await;
    let provider = factory.get_provider(&source_type).ok_or_else(|| {
        crate::ErrorKind::CoreError(format!("No provider registered for: {source_type:?}"))
            .as_error()
    })?;

    let normalized = provider.parse_identifier(&identifier)?;
    provider
        .fetch_preview(&normalized)
        .await
        .map_err(Into::into)
}

/// Install a plugin using the provider factory.
/// Downloads the plugin from the given `download_url` (from preview),
/// extracts it, and stores it.
pub async fn install_plugin_from_provider(
    source_type: String,
    identifier: String,
    download_url: String,
    tag_name: String,
    version: String,
) -> crate::Result<String> {
    use crate::features::plugins::PluginSource;

    let locator = LazyLocator::get().await?;
    let factory = locator.get_plugin_provider_factory().await;

    let st = crate::features::plugins::PluginSourceType::from_str(&source_type).map_err(|()| {
        crate::ErrorKind::CoreError(format!("Unknown plugin source type: {source_type}")).as_error()
    })?;

    let provider = factory.get_provider(&st).ok_or_else(|| {
        crate::ErrorKind::CoreError(format!("No provider registered for: {st:?}")).as_error()
    })?;

    let normalized = provider.parse_identifier(&identifier)?;
    let zip_bytes = provider.download_plugin(&download_url).await?;
    let temp_file = write_bytes_to_temp_file(&zip_bytes)?;
    let extracted = locator
        .get_plugin_extractor()
        .await
        .extract(temp_file.path())
        .await?;
    let plugin_id = extracted.plugin_id.clone();
    locator.get_plugin_storage().await.add(extracted).await?;

    // Save source info
    match st {
        crate::features::plugins::PluginSourceType::GitHub => {
            let (owner, repo) = normalized.split_once('/').unwrap_or((&normalized, ""));
            let source = PluginSource::GitHub {
                owner: owner.to_string(),
                repo: repo.to_string(),
                current_tag: tag_name,
                current_version: version,
            };
            locator
                .get_plugin_source_storage()
                .await
                .save(&plugin_id, &source)
                .await?;
        }
        crate::features::plugins::PluginSourceType::Local => {
            locator
                .get_plugin_source_storage()
                .await
                .save(&plugin_id, &PluginSource::Local)
                .await?;
        }
    }

    // Sync registry
    sync().await?;

    Ok(plugin_id)
}
