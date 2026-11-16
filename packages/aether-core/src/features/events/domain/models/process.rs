use uuid::Uuid;

use crate::{features::instance::InstanceId, shared::serializable_command::HookSpawnError};

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
    /// Why the hook could not be run, on `PostExitHookFailed`.
    pub hook_error: Option<HookSpawnError>,
}

#[derive(Clone, Debug)]
pub enum ProcessEventType {
    Launched,
    Finished,
    /// The post-exit hook could not be started; sent after `Finished`. A hook that ran and
    /// failed is not reported: nothing waits for it.
    PostExitHookFailed,
}
