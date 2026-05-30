use aether_core_plugin_api::v0::{
    InstanceDto, PluginCheckCompatibilityParamsDto, PluginImportInstanceDto,
};

use crate::features::plugins::{PluginCheckCompatibilityParams, PluginImportInstance};

impl From<PluginImportInstance> for PluginImportInstanceDto {
    fn from(value: PluginImportInstance) -> Self {
        Self {
            importer_id: value.importer_id,
            path: value.path,
        }
    }
}

impl From<PluginCheckCompatibilityParams> for PluginCheckCompatibilityParamsDto {
    fn from(value: PluginCheckCompatibilityParams) -> Self {
        Self {
            instances: value.instances.into_iter().map(InstanceDto::from).collect(),
            check_params: value.check_params.into(),
        }
    }
}
