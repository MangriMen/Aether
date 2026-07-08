use std::path::PathBuf;
use std::str::FromStr;

use crate::{
    core::app::AetherContainer,
    features::plugins::{
        EditPluginSettings, PluginDto, PluginSettings, PluginSource, PluginSourceType,
        PluginsFeature, ProviderUpdateInfo, write_bytes_to_temp_file,
    },
};

#[tracing::instrument]
pub async fn import(paths: Vec<PathBuf>) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container.import_plugins_use_case().execute(paths).await?)
}

pub async fn sync() -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container.sync_plugins_use_case().execute().await?)
}

pub async fn list() -> crate::Result<Vec<PluginDto>> {
    let container = AetherContainer::get();
    Ok(container.list_plugins_dto_use_case().execute().await?)
}

pub async fn get(plugin_id: String) -> crate::Result<PluginDto> {
    let container = AetherContainer::get();
    Ok(container
        .get_plugin_dto_use_case()
        .execute(plugin_id)
        .await?)
}

#[tracing::instrument]
pub async fn remove(plugin_id: String) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .remove_plugin_use_case()
        .execute(plugin_id)
        .await?)
}

#[tracing::instrument]
pub async fn enable(plugin_id: String) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .enable_plugin_use_case()
        .execute(plugin_id)
        .await?)
}

#[tracing::instrument]
pub async fn force_enable(plugin_id: String) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .force_enable_plugin_use_case()
        .execute(plugin_id)
        .await?)
}

#[tracing::instrument]
pub async fn disable(plugin_id: String) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .disable_plugin_use_case()
        .execute(plugin_id)
        .await?)
}

#[tracing::instrument]
pub async fn call(plugin_id: String, _: String) -> crate::Result<()> {
    let container = AetherContainer::get();
    let reg = container.plugin_registry();
    let _plugin = reg.get(&plugin_id);
    log::debug!("Calling plugin {:?}", plugin_id);
    Ok(())
}

pub async fn get_settings(plugin_id: String) -> crate::Result<Option<PluginSettings>> {
    let container = AetherContainer::get();
    Ok(container
        .get_plugin_settings_use_case()
        .execute(plugin_id)
        .await?)
}

#[tracing::instrument]
pub async fn edit_settings(
    plugin_id: String,
    edit_settings: EditPluginSettings,
) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .edit_plugin_settings_use_case()
        .execute(plugin_id, edit_settings)
        .await?)
}

pub async fn get_api_version() -> crate::Result<semver::Version> {
    let container = AetherContainer::get();
    Ok(container
        .get_plugin_api_version_use_case()
        .execute()
        .await?)
}

// ── Provider factory API ──

pub async fn get_available_providers() -> crate::Result<Vec<PluginSourceType>> {
    let container = AetherContainer::get();
    Ok(container.plugin_provider_factory().list_source_types())
}

// ── Remote source operations (provider-agnostic) ──

pub async fn get_plugin_source(plugin_id: String) -> crate::Result<Option<PluginSource>> {
    let container = AetherContainer::get();
    Ok(container.plugin_source_storage().get(&plugin_id).await?)
}

pub async fn check_for_updates(plugin_id: String) -> crate::Result<ProviderUpdateInfo> {
    let container = AetherContainer::get();
    Ok(container
        .check_for_plugin_updates_use_case()
        .execute(&plugin_id)
        .await?)
}

pub async fn update_plugin(plugin_id: String, target_tag: Option<String>) -> crate::Result<()> {
    let container = AetherContainer::get();
    Ok(container
        .update_plugin_use_case()
        .execute(&plugin_id, target_tag.as_deref())
        .await?)
}

// ── Generic provider-based API ──

pub async fn preview_plugin_from_provider(
    source_type: String,
    identifier: String,
) -> crate::Result<crate::features::plugins::ProviderPluginPreview> {
    let st = PluginSourceType::from_str(&source_type).map_err(|()| {
        crate::ErrorKind::CoreError(format!("Unknown plugin source type: {source_type}")).as_error()
    })?;

    let container = AetherContainer::get();
    let factory = container.plugin_provider_factory();
    let provider = factory.get_provider(&st).ok_or_else(|| {
        crate::ErrorKind::CoreError(format!("No provider registered for: {st:?}")).as_error()
    })?;

    let normalized = provider.parse_identifier(&identifier)?;
    provider
        .fetch_preview(&normalized)
        .await
        .map_err(Into::into)
}

pub async fn install_plugin_from_provider(
    source_type: String,
    identifier: String,
    download_url: String,
    tag_name: String,
    version: String,
) -> crate::Result<String> {
    let st = PluginSourceType::from_str(&source_type).map_err(|()| {
        crate::ErrorKind::CoreError(format!("Unknown plugin source type: {source_type}")).as_error()
    })?;

    let container = AetherContainer::get();
    let factory = container.plugin_provider_factory();
    let provider = factory.get_provider(&st).ok_or_else(|| {
        crate::ErrorKind::CoreError(format!("No provider registered for: {st:?}")).as_error()
    })?;

    let normalized = provider.parse_identifier(&identifier)?;
    let zip_bytes = provider.download_plugin(&download_url).await?;
    let temp_file = write_bytes_to_temp_file(&zip_bytes)?;
    let extracted = container
        .plugin_extractor()
        .extract(temp_file.path())
        .await?;
    let plugin_id = extracted.plugin_id.clone();
    container.plugin_storage().add(extracted).await?;

    let source = PluginSource::Remote {
        source_type: st.clone(),
        identifier: normalized,
        current_tag: tag_name,
        current_version: version,
    };
    container
        .plugin_source_storage()
        .save(&plugin_id, &source)
        .await?;

    sync().await?;

    Ok(plugin_id)
}
