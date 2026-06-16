use aether_core_plugin_api::v0::PluginInternalEventDto;

use crate::features::plugins::PluginInternalEvent;

impl From<&PluginInternalEvent> for PluginInternalEventDto {
    fn from(event: &PluginInternalEvent) -> Self {
        match event {
            PluginInternalEvent::Loaded => Self::Loaded,
            PluginInternalEvent::Unloaded => Self::Unloaded,
            PluginInternalEvent::BeforeInstanceLaunch { instance_id } => {
                Self::BeforeInstanceLaunch {
                    instance_id: instance_id.clone(),
                }
            }
            PluginInternalEvent::AfterInstanceLaunch { instance_id } => Self::AfterInstanceLaunch {
                instance_id: instance_id.clone(),
            },
        }
    }
}
