use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use path_slash::PathBufExt;

use crate::features::plugins::{PathMapping, PluginError, PluginSettings, PluginSettingsStorage};

use super::ports::EditPluginSettingsUseCasePort;

#[derive(Debug, Default)]
pub struct EditPluginSettings {
    pub allowed_hosts: Option<Vec<String>>,
    pub allowed_paths: Option<Vec<PathMapping>>,
}

pub struct EditPluginSettingsUseCase {
    plugin_settings_storage: Arc<dyn PluginSettingsStorage>,
}

impl EditPluginSettingsUseCase {
    pub fn new(plugin_settings_storage: Arc<dyn PluginSettingsStorage>) -> Self {
        Self {
            plugin_settings_storage,
        }
    }

    pub async fn execute(
        &self,
        plugin_id: String,
        edit_settings: EditPluginSettings,
    ) -> Result<(), PluginError> {
        let current = self
            .plugin_settings_storage
            .get(&plugin_id)
            .await?
            .unwrap_or_default();
        let merged = apply_edit_changes(current, &edit_settings);

        self.plugin_settings_storage
            .upsert(&plugin_id, &merged)
            .await
    }
}

fn apply_edit_changes(
    mut settings: PluginSettings,
    edit_settings: &EditPluginSettings,
) -> PluginSettings {
    if let Some(allowed_hosts) = &edit_settings.allowed_hosts {
        settings.allowed_hosts.clone_from(allowed_hosts);
    }

    if let Some(allowed_paths) = &edit_settings.allowed_paths {
        let filtered = allowed_paths
            .iter()
            .filter_map(|PathMapping(host, plugin)| {
                if !PathBuf::from(host).exists() {
                    return None;
                }

                Some(PathMapping(
                    host.to_owned(),
                    PathBuf::from(plugin.to_slash_lossy().to_string()),
                ))
            })
            .collect();

        settings.allowed_paths = filtered;
    }

    settings
}

#[async_trait]
impl EditPluginSettingsUseCasePort for EditPluginSettingsUseCase {
    async fn execute(
        &self,
        plugin_id: String,
        edit_settings: EditPluginSettings,
    ) -> Result<(), PluginError> {
        self.execute(plugin_id, edit_settings).await
    }
}
