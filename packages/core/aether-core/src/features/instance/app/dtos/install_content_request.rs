use crate::features::instance::domain::{InstanceError, ProviderId, VersionPayload};

/// Input for the unified `InstallContentUseCase`.
///
/// This is the single entry point for installing ANY content —
/// both single assets (mods, resource packs, shaders) and full modpacks.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone)]
pub struct InstallContentRequest {
    /// Which content source to use (resolved from `CapabilityRegistry<Arc<dyn ContentSource>>`).
    pub source_id: ProviderId,
    /// The content item ID to install.
    pub content_id: String,
    /// The specific version ID to install.
    pub version_id: String,
    /// Optional target instance.
    ///
    /// - For `VersionPayload::Asset`: MUST be `Some` (an instance is required).
    /// - For `VersionPayload::Modpack`: if `None`, a new instance will be created.
    pub target_instance_id: Option<String>,
}

impl InstallContentRequest {
    /// Validate that the request is well-formed for the given payload type.
    pub fn validate_for_payload(&self, payload: &VersionPayload) -> Result<(), InstanceError> {
        match payload {
            VersionPayload::Asset(_) => {
                if self.target_instance_id.is_none() {
                    return Err(InstanceError::UnsupportedOperation(
                        "target_instance_id is required for asset installation".into(),
                    ));
                }
            }
            VersionPayload::Modpack(_) => {
                // Modpack can work with or without an instance
            }
        }
        Ok(())
    }
}
