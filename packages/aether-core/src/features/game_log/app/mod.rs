use std::time::Duration;

use tokio::{
    process::{ChildStderr, ChildStdout},
    sync::oneshot,
};
use uuid::Uuid;

use crate::features::instance::InstanceId;

pub struct CaptureGameOutputParams {
    pub instance_id: InstanceId,
    pub process_id: Uuid,
    pub stdout: Option<ChildStdout>,
    pub stderr: Option<ChildStderr>,
    /// Secrets that must not reach the log (the account's access token).
    pub secrets: Vec<String>,
}

/// Resolves once the capture has written and published the last line of the game's output.
pub struct CaptureHandle {
    done: oneshot::Receiver<()>,
}

impl CaptureHandle {
    pub fn new(done: oneshot::Receiver<()>) -> Self {
        Self { done }
    }

    /// Waits for the last line, but not for ever: a wrapper command or a grandchild that
    /// inherited the pipe can hold it open after the game has exited.
    pub async fn wait(self, timeout: Duration) {
        let _ = tokio::time::timeout(timeout, self.done).await;
    }
}

/// Reads the output of a started game process until its pipes close. Capturing never fails
/// the launch: if the log cannot be written the output is still drained, or the game would
/// block on a full pipe.
pub trait GameOutputCaptureService: Send + Sync {
    fn start(&self, params: CaptureGameOutputParams) -> CaptureHandle;
}
