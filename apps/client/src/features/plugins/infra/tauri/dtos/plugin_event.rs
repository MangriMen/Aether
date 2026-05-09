use aether_core::features::events::PluginEvent;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PluginEventDto {
    Add { plugin_id: String },
    Edit { plugin_id: String },
    Remove { plugin_id: String },
    Sync,
}

impl From<PluginEvent> for PluginEventDto {
    fn from(value: PluginEvent) -> Self {
        match value {
            PluginEvent::Add { plugin_id } => Self::Add { plugin_id },
            PluginEvent::Edit { plugin_id } => Self::Edit { plugin_id },
            PluginEvent::Remove { plugin_id } => Self::Remove { plugin_id },
            PluginEvent::Sync => Self::Sync,
        }
    }
}
