use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityEntry<C: Send + Sync + Clone> {
    pub plugin_id: String,
    pub capability: C,
}
