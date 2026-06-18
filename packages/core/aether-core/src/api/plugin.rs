use std::{path::PathBuf, sync::Arc};

use crate::{
    core::LazyLocator,
    features::plugins::{
        DisablePluginUseCase, EditPluginSettings, EditPluginSettingsUseCase, EnablePluginUseCase,
        GetPluginApiVersionUseCase, GetPluginDtoUseCase, GetPluginSettingsUseCase,
        ImportPluginsUseCase, ListPluginsDtoUseCase, PluginDto, PluginSettings,
        RemovePluginUseCase, SyncPluginsUseCase,
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
