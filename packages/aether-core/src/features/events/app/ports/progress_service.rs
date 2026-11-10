use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::features::events::{
    EventError, ProgressBarId, ProgressEventType, ProgressStage, SharedDownload, SharedProgressWait,
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

    /// Starts a bar whose work the user may call off ([`cancel_progress`](Self::cancel_progress)):
    /// the token fires when they do, until the work passes its point of no return
    /// ([`close_cancellation`](Self::close_cancellation)). The token goes with the bar.
    ///
    /// Calling off is extra detail, so by default the bar is started without it and the token
    /// never fires.
    async fn init_cancellable_progress(
        &self,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<(ProgressBarId, CancellationToken), EventError> {
        let bar = self.init_progress(event_type, total, message).await?;
        Ok((bar, CancellationToken::new()))
    }

    /// Calls off the work of the bar `progress_bar_id`: fires its token and tells listeners.
    /// Nothing for a bar that is gone (its work ended meanwhile), that cannot be called off, or
    /// whose work is past its point of no return or called off already.
    async fn cancel_progress(&self, _progress_bar_id: Uuid) -> Result<(), EventError> {
        Ok(())
    }

    /// Marks the point of no return of the bar's work: from now on
    /// [`cancel_progress`](Self::cancel_progress) does not fire its token, so a token not fired
    /// by the time this returns never is. Listeners are told the work can no longer be called
    /// off.
    async fn close_cancellation(&self, _progress_bar_id: &ProgressBarId) -> Result<(), EventError> {
        Ok(())
    }

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
    /// a game version): at the top, of no install's, and known by `download`. Every bar
    /// waiting on `download` ([`wait_on_shared`](Self::wait_on_shared)) gets a mirror of it.
    ///
    /// Sharing is extra detail, so by default the bar is started on its own.
    async fn init_shared_progress(
        &self,
        download: SharedDownload,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        self.init_progress(download.event_type(), total, message)
            .await
    }

    /// Starts the shared bar of a download that the user may call off, as
    /// [`init_cancellable_progress`](Self::init_cancellable_progress) does a bar of its own.
    ///
    /// Calling off is extra detail, so by default the bar is started without it and the token
    /// never fires.
    async fn init_cancellable_shared_progress(
        &self,
        download: SharedDownload,
        total: f64,
        message: String,
    ) -> Result<(ProgressBarId, CancellationToken), EventError> {
        let bar = self.init_shared_progress(download, total, message).await?;
        Ok((bar, CancellationToken::new()))
    }

    /// Notes that the bar `waiting` waits on the shared download `download`, until the guard
    /// is dropped. Meanwhile, `waiting` gets a mirror of the shared bar of `download` running
    /// now, if any, and of every one started later: a bar under it that shows what the shared
    /// bar shows, is not counted in its bytes, and ends with the shared bar. A mirror whose
    /// parent ends first ends with it, cancelled; the shared bar goes on. Without a bar to wait
    /// with, the wait tracks nothing.
    ///
    /// Sharing is extra detail, so by default nothing is mirrored.
    async fn wait_on_shared(
        &self,
        _waiting: Option<&ProgressBarId>,
        _download: SharedDownload,
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
