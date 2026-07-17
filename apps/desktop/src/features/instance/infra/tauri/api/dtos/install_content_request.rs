use aether_core::features::instance::InstallContentRequest;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::instance::infra::ProviderIdDto;

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_field_names)]
pub struct InstallContentRequestDto {
    /// Which content source to use (`plugin_id`#`capability_id`).
    pub source_id: ProviderIdDto,
    /// The content item ID to install.
    pub content_id: String,
    /// The specific version ID to install.
    pub version_id: String,
    /// Optional target instance ID.
    /// For asset payloads: required.
    /// For modpack payloads: if None, a new instance will be created.
    #[specta(optional)]
    pub target_instance_id: Option<String>,
}

impl From<InstallContentRequestDto> for InstallContentRequest {
    fn from(dto: InstallContentRequestDto) -> Self {
        Self {
            source_id: dto.source_id.into(),
            content_id: dto.content_id,
            version_id: dto.version_id,
            target_instance_id: dto.target_instance_id,
        }
    }
}
