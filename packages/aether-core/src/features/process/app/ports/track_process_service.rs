use std::process::ExitStatus;

use async_trait::async_trait;

use crate::features::process::app::use_cases::TrackProcessParams;

#[async_trait]
pub trait TrackProcessService: Send + Sync {
    /// `None` when the exit status could not be read: the OS refused the wait, or the game was
    /// adopted on a platform that tells only its parent how it ended.
    async fn execute(&self, params: TrackProcessParams) -> Option<ExitStatus>;
}
