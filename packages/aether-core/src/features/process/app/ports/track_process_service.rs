use std::process::ExitStatus;

use async_trait::async_trait;

use crate::features::process::app::use_cases::TrackProcessParams;

#[async_trait]
pub trait TrackProcessService: Send + Sync {
    /// `None` when the exit status could not be read (the OS refused the wait).
    async fn execute(&self, params: TrackProcessParams) -> Option<ExitStatus>;
}
