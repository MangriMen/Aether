use std::time::Instant;

use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::transfer_window::TransferWindow;
use super::{CancelState, ProgressEvent, ProgressEventType, ProgressStage, ProgressStatus};

#[derive(Debug, Clone)]
pub struct ProgressBar {
    // Identifies the bar; the live values are in the storage, not in copies of this struct.
    pub id: Uuid,
    pub message: String,
    pub total: f64,
    pub current: f64,
    pub last_sent: f64,
    pub progress_type: ProgressEventType,
    pub stage: Option<ProgressStage>,
    /// `Failed` once `ProgressBarId::fail` is called, `Cancelled` once
    /// `ProgressBarId::cancel` is; the bar is gone when it is done.
    pub status: ProgressStatus,
    /// The bar this one is a part of. A parent counts the bytes of its children as its own, and
    /// takes the children still there with it when it is done.
    pub parent_id: Option<Uuid>,
    /// The shared bar this one mirrors, when it is a mirror: it shows what the shared bar
    /// shows, is not counted in its parent's bytes, and ends with the shared bar (or its own
    /// parent, whichever is first).
    pub mirror_of: Option<Uuid>,
    /// Whether a bar was ever started under this one: only then does finishing it look for
    /// children left behind.
    pub(crate) has_children: bool,
    /// Whether this is the bar of a shared download, which bars waiting on it mirror.
    pub(crate) shared: bool,
    /// The mirrors of this shared bar; some may be gone already, ended with their parents.
    pub(crate) mirrors: Vec<Uuid>,
    pub bytes_done: Option<u64>,
    pub bytes_total: Option<u64>,
    /// Transfer speed in bytes per second.
    pub speed: Option<f64>,
    /// The recent history of `bytes_done`, which `speed` is measured over.
    pub(crate) transfer: TransferWindow,
    /// When the bar last sent an event; events for bytes alone are spaced out from it.
    pub(crate) last_sent_at: Option<Instant>,
    /// How the user calls the bar's work off, when it can be.
    pub(crate) cancel: Option<BarCancel>,
}

/// The bar's side of calling its work off: the token the work watches, and whether the work is
/// past the point where it could stop. Changed only inside the storage's atomic update of the
/// bar, so a call-off and the point of no return are never both taken.
#[derive(Debug, Clone)]
pub(crate) struct BarCancel {
    token: CancellationToken,
    closed: bool,
}

impl BarCancel {
    pub(crate) fn new(token: CancellationToken) -> Self {
        Self {
            token,
            closed: false,
        }
    }

    pub(crate) fn state(&self) -> CancelState {
        if self.token.is_cancelled() {
            CancelState::Requested
        } else if self.closed {
            CancelState::Closed
        } else {
            CancelState::Open
        }
    }

    /// Fire the token, unless the work is past the point of no return or called off already.
    /// Whether this call fired it.
    pub(crate) fn request(&mut self) -> bool {
        if self.closed || self.token.is_cancelled() {
            return false;
        }
        self.token.cancel();
        true
    }

    /// Pass the point of no return: the token is not fired from now on. Whether the bar
    /// shows anything new for it (a call-off that came first stays shown).
    pub(crate) fn close(&mut self) -> bool {
        let was = self.state();
        self.closed = true;
        self.state() != was
    }
}

impl ProgressBar {
    /// A new running bar, at nothing done yet.
    pub(crate) fn new(
        id: Uuid,
        progress_type: ProgressEventType,
        total: f64,
        message: String,
        parent_id: Option<Uuid>,
    ) -> Self {
        Self {
            id,
            message,
            total,
            current: 0.0,
            last_sent: 0.0,
            progress_type,
            stage: None,
            status: ProgressStatus::Running,
            parent_id,
            mirror_of: None,
            has_children: false,
            shared: false,
            mirrors: Vec::new(),
            bytes_done: None,
            bytes_total: None,
            speed: None,
            transfer: TransferWindow::default(),
            last_sent_at: None,
            cancel: None,
        }
    }

    /// Whether the user may call the bar's work off; `None` for work that cannot be.
    pub fn cancel_state(&self) -> Option<CancelState> {
        self.cancel.as_ref().map(BarCancel::state)
    }

    /// A mirror of the shared bar `shared` under the bar `parent_id`, showing what it shows.
    pub(crate) fn mirror(id: Uuid, shared: &ProgressBar, parent_id: Uuid) -> Self {
        let mut mirror = Self::new(
            id,
            shared.progress_type.clone(),
            shared.total,
            shared.message.clone(),
            Some(parent_id),
        );
        mirror.mirror_of = Some(shared.id);
        mirror.show(&shared.shown());
        mirror
    }

    /// What this bar shows, for its mirrors to take on.
    pub(crate) fn shown(&self) -> Shown {
        Shown {
            message: self.message.clone(),
            total: self.total,
            current: self.current,
            last_sent: self.last_sent,
            stage: self.stage,
            bytes_done: self.bytes_done,
            bytes_total: self.bytes_total,
            speed: self.speed,
            last_sent_at: self.last_sent_at,
        }
    }

    /// Take on what a shared bar shows: how far it is, its bytes, speed and stage.
    pub(crate) fn show(&mut self, shown: &Shown) {
        self.message.clone_from(&shown.message);
        self.total = shown.total;
        self.current = shown.current;
        self.last_sent = shown.last_sent;
        self.stage = shown.stage;
        self.bytes_done = shown.bytes_done;
        self.bytes_total = shown.bytes_total;
        self.speed = shown.speed;
        self.last_sent_at = shown.last_sent_at;
    }

    /// Measure `speed` up to `now`. There is none once all the expected bytes are in.
    pub(crate) fn refresh_speed(&mut self, now: Instant) {
        let finished = self
            .bytes_total
            .is_some_and(|total| self.bytes_done.unwrap_or(0) >= total);
        self.speed = if finished {
            None
        } else {
            self.transfer.speed(now)
        };
    }

    /// An event describing this bar with the given fraction, message and status.
    pub(crate) fn event(
        &self,
        fraction: Option<f64>,
        message: String,
        status: ProgressStatus,
    ) -> ProgressEvent {
        ProgressEvent {
            event: self.progress_type.clone(),
            progress_bar_id: self.id,
            fraction,
            message,
            stage: self.stage,
            status,
            parent_id: self.parent_id,
            mirror_of: self.mirror_of,
            bytes_done: self.bytes_done,
            bytes_total: self.bytes_total,
            speed: self.speed,
            cancel: self.cancel_state(),
        }
    }

    /// A running event of this bar with `message`; the fraction is left out once the bar is
    /// full.
    pub(crate) fn running_event(&self, message: String) -> ProgressEvent {
        let fraction = self.current / self.total;
        let fraction = if fraction >= 1.0 {
            None
        } else {
            Some(fraction)
        };
        self.event(fraction, message, ProgressStatus::Running)
    }

    /// The last event of a bar: done or, after `ProgressBarId::fail` or
    /// `ProgressBarId::cancel`, failed or cancelled.
    pub(crate) fn finished_event(&self) -> ProgressEvent {
        match &self.status {
            ProgressStatus::Failed(_) => {
                self.event(None, FAILED_MESSAGE.to_string(), self.status.clone())
            }
            ProgressStatus::Cancelled => self.event(
                None,
                CANCELLED_MESSAGE.to_string(),
                ProgressStatus::Cancelled,
            ),
            ProgressStatus::Running | ProgressStatus::Done => {
                self.event(None, COMPLETED_MESSAGE.to_string(), ProgressStatus::Done)
            }
        }
    }

    /// Mark a bar that is left behind by the end of its parent as cancelled, unless it has
    /// failed already.
    pub(crate) fn cut_short(&mut self) {
        if self.status == ProgressStatus::Running {
            self.status = ProgressStatus::Cancelled;
        }
    }
}

/// What a shared bar shows ([`ProgressBar::shown`]), taken without copying the whole bar, for
/// its mirrors to take on ([`ProgressBar::show`]).
#[derive(Debug, Clone)]
pub(crate) struct Shown {
    message: String,
    total: f64,
    current: f64,
    last_sent: f64,
    stage: Option<ProgressStage>,
    bytes_done: Option<u64>,
    bytes_total: Option<u64>,
    speed: Option<f64>,
    last_sent_at: Option<Instant>,
}

const COMPLETED_MESSAGE: &str = "Completed";
/// The text of a failed bar; the error itself is in its status.
pub(crate) const FAILED_MESSAGE: &str = "Failed";
const CANCELLED_MESSAGE: &str = "Cancelled";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorKind;
    use crate::features::events::ProgressFailure;

    /// A failure for a test, by its text.
    fn failure(text: &str) -> ErrorKind {
        ErrorKind::CoreError(text.to_owned())
    }

    fn jar_event() -> ProgressEventType {
        ProgressEventType::JavaDownload { version: 21 }
    }

    fn bar(id: Uuid) -> ProgressBar {
        ProgressBar {
            current: 4.0,
            ..ProgressBar::new(id, jar_event(), 10.0, "Downloading".to_string(), None)
        }
    }

    #[test]
    fn a_bar_cut_short_ends_cancelled_unless_it_failed() {
        let mut running = bar(Uuid::new_v4());
        running.cut_short();
        let event = running.finished_event();
        assert_eq!(event.status, ProgressStatus::Cancelled);
        assert_eq!(event.message, CANCELLED_MESSAGE);
        assert_eq!(event.fraction, None);

        let mut failed = bar(Uuid::new_v4());
        failed.status = ProgressStatus::Failed(ProgressFailure::new(failure("boom")));
        failed.cut_short();
        let event = failed.finished_event();
        assert!(matches!(event.status, ProgressStatus::Failed(_)));
        assert_eq!(event.message, FAILED_MESSAGE);
    }

    #[test]
    fn a_running_bar_ends_done_when_not_cut_short() {
        let event = bar(Uuid::new_v4()).finished_event();
        assert_eq!(event.status, ProgressStatus::Done);
        assert_eq!(event.message, COMPLETED_MESSAGE);
    }
}
