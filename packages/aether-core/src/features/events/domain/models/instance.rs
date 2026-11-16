use crate::features::instance::InstanceId;

#[derive(Debug, Clone)]
pub struct InstanceEvent {
    pub event: InstanceEventType,
    pub instance_id: InstanceId,
}

#[derive(Debug, Clone)]
pub enum InstanceEventType {
    Created,
    Synced,
    /// Only the metadata or icons of the instance's content were refreshed.
    ContentSynced,
    Edited,
    Removed,
    /// A launch took the instance: it may be called off until the game starts.
    LaunchStarted,
    /// The launch is over, whether the game started, it failed or it was called off.
    LaunchEnded,
}
