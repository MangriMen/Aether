use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PluginEvent {
    Add { plugin_id: String },
    Edit { plugin_id: String },
    Remove { plugin_id: String },
    Sync,
}
