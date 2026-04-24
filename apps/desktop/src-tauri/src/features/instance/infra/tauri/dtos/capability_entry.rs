use aether_core::shared::CapabilityEntry;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityEntryDto<C: Send + Sync + Clone> {
    pub plugin_id: String,
    pub capability: C,
}

impl<C: Send + Sync + Clone> From<CapabilityEntry<C>> for CapabilityEntryDto<C> {
    fn from(value: CapabilityEntry<C>) -> Self {
        CapabilityEntryDto {
            plugin_id: value.plugin_id,
            capability: value.capability,
        }
    }
}
