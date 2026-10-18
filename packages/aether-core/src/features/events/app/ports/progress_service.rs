use async_trait::async_trait;
use uuid::Uuid;

use crate::features::events::{
    EventError, ProgressBarId, ProgressEventType, ProgressStage, SharedProgressWait,
};

/// Updates of one bar may come from many tasks at once: a parent hears of the bytes of all its
/// children.
#[async_trait]
pub trait ProgressService: Send + Sync {
    async fn init_progress(
        &self,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError>;

    /// Starts a bar as a part of the bar `parent_id`: its events name the parent, the parent
    /// counts its bytes as its own, and it ends with the parent at the latest.
    ///
    /// The tree is extra detail, so by default the bar is started on its own.
    async fn init_child_progress(
        &self,
        _parent_id: Uuid,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        self.init_progress(event_type, total, message).await
    }

    /// Starts the bar of a download that every install needing the same thing waits on (a Java,
    /// a game version): at the top, of no install's, and known by `event_type`. Every bar
    /// waiting on `event_type` ([`wait_on_shared`](Self::wait_on_shared)) gets a mirror of it.
    ///
    /// Sharing is extra detail, so by default the bar is started on its own.
    async fn init_shared_progress(
        &self,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        self.init_progress(event_type, total, message).await
    }

    /// Notes that the bar `waiting` waits on the shared download `event_type`, until the guard
    /// is dropped. Meanwhile, `waiting` gets a mirror of the shared bar of `event_type` running
    /// now, if any, and of every one started later: a bar under it that shows what the shared
    /// bar shows, is not counted in its bytes, and ends with the shared bar. A mirror whose
    /// parent ends first ends with it, cancelled; the shared bar goes on. Without a bar to wait
    /// with, the wait tracks nothing.
    ///
    /// Sharing is extra detail, so by default nothing is mirrored.
    async fn wait_on_shared(
        &self,
        _waiting: Option<&ProgressBarId>,
        _event_type: ProgressEventType,
    ) -> SharedProgressWait {
        SharedProgressWait::none()
    }

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

    /// Counts `bytes` more as transferred for the bar, which also gives it a speed. Listeners
    /// hear of it a few times a second at most, always of the first bytes and of the ones that
    /// complete [`add_progress_bytes_total`](Self::add_progress_bytes_total).
    ///
    /// Bytes are extra detail next to the fraction, so by default they are ignored.
    async fn add_progress_bytes(
        &self,
        _progress_bar_id: &ProgressBarId,
        _bytes: u64,
    ) -> Result<(), EventError> {
        Ok(())
    }

    /// Adds `bytes` to how many the bar expects to transfer in all, so that transfers sharing a
    /// bar each add their own size. Until it is called the total is unknown; it tells listeners
    /// nothing by itself, the next event carries it.
    ///
    /// Bytes are extra detail next to the fraction, so by default they are ignored.
    async fn add_progress_bytes_total(
        &self,
        _progress_bar_id: &ProgressBarId,
        _bytes: u64,
    ) -> Result<(), EventError> {
        Ok(())
    }
}
