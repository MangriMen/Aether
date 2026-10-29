use crate::shared::capability::PluginId;

#[derive(Debug, Clone)]
pub enum PluginEvent {
    Add { plugin_id: PluginId },
    Edit { plugin_id: PluginId },
    Remove { plugin_id: PluginId },
    Sync,
}
