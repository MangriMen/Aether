use serde::{Deserialize, Serialize};

use super::ids::{CapabilityId, PluginId};

// --- CapabilityEntry ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityEntry<C: Send + Sync + Clone> {
    pub plugin_id: PluginId,
    pub capability: C,
}

// --- RegistryError ---

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Capability {capability_type} with id \"{capability_id}\" not found")]
    CapabilityNotFound {
        capability_type: &'static str,
        capability_id: CapabilityId,
    },
}
