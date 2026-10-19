use async_trait::async_trait;
use log::debug;
use uuid::Uuid;

use crate::features::events::{
    ProgressBarId, ProgressEventType, ProgressService, ProgressStage, SharedDownload,
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

    /// Starts a bar under `parent`, or on its own without one.
    async fn init_progress_under_safe(
        &self,
        parent: Option<&ProgressBarId>,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Option<ProgressBarId> {
        match parent {
            Some(parent) => {
                self.init_child_progress_safe(parent.id, event_type, total, message)
                    .await
            }
            None => self.init_progress_safe(event_type, total, message).await,
        }
    }

    async fn init_shared_progress_safe(
        &self,
        download: SharedDownload,
        total: f64,
        message: String,
    ) -> Option<ProgressBarId> {
        self.init_shared_progress(download, total, message)
            .await
            .map_err(|error| debug!("Failed to send progress: {error}"))
            .ok()
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
