pub struct InitializePluginsUseCase {}

impl InitializePluginsUseCase {
    pub async fn execute() -> crate::Result<()> {
        aether_core::api::plugin::sync().await?;
        Self::load_enabled_plugins().await?;
        Ok(())
    }

    async fn load_enabled_plugins() -> crate::Result<()> {
        let settings = aether_core::api::settings::get().await?;

        for plugin_id in settings.enabled_plugins() {
            if let Err(e) = aether_core::api::plugin::enable(plugin_id.clone()).await {
                log::error!("Failed to load plugin {plugin_id}: {e}");
            }
        }

        Ok(())
    }
}
