use std::sync::Arc;

use aether_core::{
    core::app::AetherContainer,
    features::{plugins::PluginsFeature, settings::SettingsFeature},
};

pub struct InitializePluginsUseCase {}

impl InitializePluginsUseCase {
    pub async fn execute(container: Arc<AetherContainer>) -> crate::Result<()> {
        container
            .sync_plugins_use_case()
            .execute()
            .await
            .map_err(aether_core::Error::from)?;
        Self::load_enabled_plugins(container).await?;
        Ok(())
    }

    async fn load_enabled_plugins(container: Arc<AetherContainer>) -> crate::Result<()> {
        let settings = container
            .get_settings_use_case()
            .execute()
            .await
            .map_err(aether_core::Error::from)?;

        for plugin_id in settings.enabled_plugins() {
            if let Err(e) = container
                .enable_plugin_use_case()
                .execute(plugin_id.clone())
                .await
            {
                log::error!("Failed to load plugin {plugin_id}: {e}");
            }
        }

        Ok(())
    }
}
