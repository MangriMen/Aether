use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProcessMetadata {
    uuid: Uuid,
    instance_id: String,
    start_time: DateTime<Utc>,
}

impl MinecraftProcessMetadata {
    pub fn new(instance_id: String) -> Self {
        MinecraftProcessMetadata {
            uuid: Uuid::new_v4(),
            instance_id,
            start_time: Utc::now(),
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
    pub fn start_time(&self) -> &DateTime<Utc> {
        &self.start_time
    }
}
