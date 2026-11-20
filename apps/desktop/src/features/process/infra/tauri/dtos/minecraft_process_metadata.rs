use aether_core::features::process::{MinecraftProcessMetadata, ProcessOrigin};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftProcessMetadataDto {
    uuid: Uuid,
    instance_id: String,
    start_time: DateTime<Utc>,
    /// Started by an earlier launcher: its console output went with that one.
    adopted: bool,
}

impl From<MinecraftProcessMetadata> for MinecraftProcessMetadataDto {
    fn from(value: MinecraftProcessMetadata) -> Self {
        MinecraftProcessMetadataDto {
            uuid: value.uuid(),
            instance_id: value.instance_id().to_string(),
            start_time: *value.start_time(),
            adopted: value.origin() == ProcessOrigin::Adopted,
        }
    }
}
