use aether_core::features::process::MinecraftProcessMetadata;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MinecraftProcessMetadataDto {
    uuid: Uuid,
    instance_id: String,
    start_time: DateTime<Utc>,
}

impl From<MinecraftProcessMetadata> for MinecraftProcessMetadataDto {
    fn from(value: MinecraftProcessMetadata) -> Self {
        MinecraftProcessMetadataDto {
            uuid: value.uuid(),
            instance_id: value.instance_id().to_owned(),
            start_time: *value.start_time(),
        }
    }
}
