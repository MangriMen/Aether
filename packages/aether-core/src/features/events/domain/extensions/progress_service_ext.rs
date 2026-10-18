use async_trait::async_trait;
use log::debug;
use uuid::Uuid;

use crate::features::events::{
    ProgressBarId, ProgressEventType, ProgressService, ProgressStage,
    domain::models::progress_parent::current_progress_parent,
};

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

    async fn init_child_progress_safe(
        &self,
        parent_id: Uuid,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Option<ProgressBarId> {
        self.init_child_progress(parent_id, event_type, total, message)
            .await
            .map_err(|error| debug!("Failed to send progress: {error}"))
            .ok()
    }

    /// Starts a bar under the parent of the surrounding
    /// [`with_progress_parent`](crate::features::events::with_progress_parent), or on its own
    /// outside of one.
    async fn init_progress_in_scope_safe(
        &self,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Option<ProgressBarId> {
        match current_progress_parent() {
            Some(parent_id) => {
                self.init_child_progress_safe(parent_id, event_type, total, message)
                    .await
            }
            None => self.init_progress_safe(event_type, total, message).await,
        }
    }

    async fn emit_progress_safe(&self, id: &ProgressBarId, progress: f64, message: Option<&str>) {
        if let Err(e) = self.emit_progress(id, progress, message).await {
            debug!("Failed to send progress: {e}");
        }
    }

    async fn set_progress_stage_safe(&self, id: &ProgressBarId, stage: Option<ProgressStage>) {
        if let Err(e) = self.set_progress_stage(id, stage).await {
            debug!("Failed to send progress: {e}");
        }
    }

    async fn add_progress_bytes_safe(&self, id: &ProgressBarId, bytes: u64) {
        if let Err(e) = self.add_progress_bytes(id, bytes).await {
            debug!("Failed to send progress: {e}");
        }
    }

    async fn add_progress_bytes_total_safe(&self, id: &ProgressBarId, bytes: u64) {
        if let Err(e) = self.add_progress_bytes_total(id, bytes).await {
            debug!("Failed to send progress: {e}");
        }
    }
}

impl<T: ?Sized + ProgressService> ProgressServiceExt for T {}
