use aether_core_plugin_api::v0::PluginImportInstanceDto;

use crate::features::plugins::PluginImportInstance;

impl From<PluginImportInstance> for PluginImportInstanceDto {
    fn from(value: PluginImportInstance) -> Self {
        Self {
            importer_id: value.importer_id,
            path: value.path,
        }
    }
}
