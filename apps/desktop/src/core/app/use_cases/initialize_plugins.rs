use std::sync::Arc;

use aether_core::{
    core::app::AetherContainer,
    features::{
        instance::PluginId,
        plugins::{PluginError, PluginsFeature},
        settings::SettingsFeature,
    },
};

pub struct InitializePluginsUseCase {}

impl InitializePluginsUseCase {
    pub async fn execute(container: Arc<AetherContainer>) -> crate::Result<()> {
        container
            .sync_plugins_service()
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
            // Stored by an older launcher, or by hand: no plugin can have such an id.
            let Ok(id) = PluginId::try_from(plugin_id.as_str()) else {
                tracing::warn!("Skipping enabled plugin '{plugin_id}': not a plugin id");
                continue;
            };
            match container.enable_plugin_use_case().execute(id).await {
                Ok(()) => {}
                Err(e) if is_already_loaded(&e) => {
                    tracing::debug!("Plugin {plugin_id} is already loaded");
                }
                Err(e) => tracing::error!("Failed to load plugin {plugin_id}: {e}"),
            }
        }

        Ok(())
    }
}

fn is_already_loaded(error: &PluginError) -> bool {
    matches!(error, PluginError::AlreadyLoaded { .. })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_treat_already_loaded_as_normal_and_other_errors_as_failures() {
        let id = PluginId::try_from("test-plugin").unwrap();

        assert!(is_already_loaded(&PluginError::AlreadyLoaded {
            plugin_id: id.clone()
        }));
        assert!(!is_already_loaded(&PluginError::LoadingInProgress {
            plugin_id: id
        }));
    }
}
