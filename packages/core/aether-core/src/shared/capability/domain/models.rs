use serde::{Deserialize, Serialize};
use serr::SerializeError;

// --- CapabilityEntry ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityEntry<C: Send + Sync + Clone> {
    pub plugin_id: String,
    pub capability: C,
}

// --- RegistryError ---

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum RegistryError {
    #[error("Capability {capability_type} with id \"{capability_id}\" not found")]
    CapabilityNotFound {
        capability_type: &'static str,
        capability_id: String,
    },
}
