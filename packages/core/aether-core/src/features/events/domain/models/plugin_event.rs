use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PluginEvent {
    pub event: PluginEventType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginEventType {
    Add { plugin_id: String },
    Edit { plugin_id: String },
    Remove { plugin_id: String },
    Sync,
}
