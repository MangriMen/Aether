use uuid::Uuid;

use crate::features::instance::InstanceId;

#[derive(Clone, Debug)]
pub struct ProcessEvent {
    pub instance_id: InstanceId,
    pub process_id: Uuid,
    pub event: ProcessEventType,
    pub message: String,
    /// Set on `Finished` when the exit status is known.
    pub exit_code: Option<i32>,
    /// The game ended on its own with a failure; a stop requested by the user is not a crash.
    pub crashed: bool,
}

#[derive(Clone, Debug)]
pub enum ProcessEventType {
    Launched,
    Finished,
}
