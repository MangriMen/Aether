use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstanceEvent {
    pub event: InstanceEventType,
    pub instance_id: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InstanceEventType {
    Created,
    Synced,
    Edited,
    Removed,
}
