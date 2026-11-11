use aether_core_plugin_api::v0::{InstanceSummaryDto, PluginCheckCompatibilityParamsDto};

use crate::features::plugins::PluginCheckCompatibilityParams;

impl From<PluginCheckCompatibilityParams> for PluginCheckCompatibilityParamsDto {
    fn from(value: PluginCheckCompatibilityParams) -> Self {
        Self {
            instances: value
                .instances
                .into_iter()
                .map(InstanceSummaryDto::from)
                .collect(),
            check_params: value.check_params.into(),
        }
    }
}
