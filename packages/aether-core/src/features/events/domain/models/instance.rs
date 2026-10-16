#[derive(Debug, Clone)]
pub struct InstanceEvent {
    pub event: InstanceEventType,
    pub instance_id: String,
}

#[derive(Debug, Clone)]
pub enum InstanceEventType {
    Created,
    Synced,
    /// Only the metadata or icons of the instance's content were refreshed.
    ContentSynced,
    Edited,
    Removed,
}
