use async_trait::async_trait;

use crate::features::events::{EventError, ProgressBarId, ProgressEventType, ProgressStage};

#[async_trait]
pub trait ProgressService: Send + Sync {
    async fn init_progress(
        &self,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError>;

    async fn init_or_edit_progress(
        &self,
        progress_bar_id: Option<ProgressBarId>,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError>;

    async fn emit_progress(
        &self,
        progress_bar_id: &ProgressBarId,
        increment_frac: f64,
        message: Option<&str>,
    ) -> Result<(), EventError>;

    async fn edit_progress(
        &self,
        progress_bar_id: &ProgressBarId,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<(), EventError>;

    /// Sets what the bar is busy with (`None` clears it) and tells listeners right away;
    /// later events of the bar carry the stage too.
    ///
    /// The stage is extra detail next to the message, so by default it is ignored.
    async fn set_progress_stage(
        &self,
        _progress_bar_id: &ProgressBarId,
        _stage: Option<ProgressStage>,
    ) -> Result<(), EventError> {
        Ok(())
    }
}
