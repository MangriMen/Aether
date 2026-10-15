use std::process::ExitStatus;

use async_trait::async_trait;

use crate::features::process::app::use_cases::TrackProcessParams;

#[async_trait]
pub trait TrackProcessService: Send + Sync {
    async fn execute(&self, params: TrackProcessParams) -> ExitStatus;
}
