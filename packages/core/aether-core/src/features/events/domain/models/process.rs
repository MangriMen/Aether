use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProcessEvent {
    pub instance_id: String,
    pub process_id: Uuid,
    pub event: ProcessEventType,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ProcessEventType {
    Launched,
    Finished,
}
