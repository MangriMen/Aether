use async_trait::async_trait;
use log::debug;

use crate::features::events::{ProgressBarId, ProgressEventType, ProgressService};

#[async_trait]
pub trait ProgressServiceExt: ProgressService {
    async fn init_progress_safe(
        &self,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Option<ProgressBarId> {
        self.init_progress(event_type, total, message)
            .await
            .map_err(|error| debug!("Failed to send progress: {error}"))
            .ok()
    }

    async fn emit_progress_safe(&self, id: &ProgressBarId, progress: f64, message: Option<&str>) {
        if let Err(e) = self.emit_progress(id, progress, message).await {
            debug!("Failed to send progress: {e}");
        }
    }
}

impl<T: ProgressService> ProgressServiceExt for T {}
