use serde::{Deserialize, Serialize};

// --- CapabilityEntry ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityEntry<C: Send + Sync + Clone> {
    pub plugin_id: String,
    pub capability: C,
}

// --- RegistryError ---

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Capability {capability_type} with id \"{capability_id}\" not found")]
    CapabilityNotFound {
        capability_type: &'static str,
        capability_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_not_found_format() {
        let err = RegistryError::CapabilityNotFound {
            capability_type: "content_provider",
            capability_id: "my_plugin".into(),
        };
        assert_eq!(
            err.to_string(),
            "Capability content_provider with id \"my_plugin\" not found"
        );
    }
}
