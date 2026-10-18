use uuid::Uuid;

use crate::features::instance::InstanceId;

#[derive(Clone, Debug)]
pub struct ProcessEvent {
    pub instance_id: InstanceId,
    pub process_id: Uuid,
    pub event: ProcessEventType,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum ProcessEventType {
    Launched,
    Finished,
}
