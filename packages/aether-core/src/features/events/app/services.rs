use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::events::{
    EventError, ProgressBar, ProgressBarId, ProgressBarStorage, ProgressBarStorageExt,
    ProgressEventType, ProgressStage, ProgressStatus, SharedEventEmitter,
    app::ports::ProgressService, domain::TransferWindow,
};

/// The least time between an event of a bar and one sent for its bytes alone: four a second
/// keep the numbers of a large file moving without an event for every chunk of a small one.
const BYTES_EVENT_INTERVAL: Duration = Duration::from_millis(250);

/// Where the service reads the time from: the speed of a transfer and the spacing of its events
/// are measured with it.
trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}

struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

pub struct ProgressServiceImpl<PS: ProgressBarStorage + 'static> {
    pub event_emitter: SharedEventEmitter,
    progress_storage: Arc<PS>,
    clock: Arc<dyn Clock>,
}

impl<PS: ProgressBarStorage + 'static> ProgressServiceImpl<PS> {
    pub fn new(event_emitter: SharedEventEmitter, progress_storage: Arc<PS>) -> Self {
        Self {
            event_emitter,
            progress_storage,
            clock: Arc::new(SystemClock),
        }
    }

    #[cfg(test)]
    fn with_clock(
        event_emitter: SharedEventEmitter,
        progress_storage: Arc<PS>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            event_emitter,
            progress_storage,
            clock,
        }
    }

    /// Send the bar as it is at `now` with `message`, and note what was sent.
    async fn emit_progress_inner(
        &self,
        progress_bar: &mut ProgressBar,
        message: String,
        now: Instant,
    ) -> Result<(), EventError> {
        progress_bar.refresh_speed(now);
        let display_frac = progress_bar.current / progress_bar.total;

        self.event_emitter
            .emit(
                progress_bar
                    .event(
                        shown_fraction(display_frac),
                        message,
                        ProgressStatus::Running,
                    )
                    .into(),
            )
            .await?;

        progress_bar.last_sent = display_frac;
        progress_bar.last_sent_at = Some(now);
        Ok(())
    }
}

/// The fraction shown for `display_frac`: `None` once the bar is full.
fn shown_fraction(display_frac: f64) -> Option<f64> {
    if display_frac >= 1.0 {
        None
    } else {
        Some(display_frac)
    }
}

#[async_trait]
impl<PS: ProgressBarStorage + 'static> ProgressService for ProgressServiceImpl<PS> {
    async fn init_progress(
        &self,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        let id = Uuid::new_v4();
        let progress_bar_id = ProgressBarId {
            id,
            progress_storage: Some(self.progress_storage.clone()),
            event_emitter: Some(self.event_emitter.clone()),
        };

        self.progress_storage
            .insert(
                id,
                ProgressBar {
                    id,
                    message,
                    total,
                    current: 0.0,
                    last_sent: 0.0,
                    progress_type: event_type,
                    stage: None,
                    status: ProgressStatus::Running,
                    parent_id: None,
                    bytes_done: None,
                    bytes_total: None,
                    speed: None,
                    transfer: TransferWindow::default(),
                    last_sent_at: None,
                },
            )
            .await?;

        self.emit_progress(&progress_bar_id, 0.0, None).await?;

        Ok(progress_bar_id)
    }

    async fn init_or_edit_progress(
        &self,
        progress_bar_id: Option<ProgressBarId>,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        if let Some(progress_bar_id) = progress_bar_id {
            self.edit_progress(&progress_bar_id, event_type, total, message)
                .await?;

            Ok(progress_bar_id)
        } else {
            self.init_progress(event_type, total, message).await
        }
    }

    async fn emit_progress(
        &self,
        progress_bar_id: &ProgressBarId,
        increment_frac: f64,
        message: Option<&str>,
    ) -> Result<(), EventError> {
        let mut progress_bar = self.progress_storage.get(progress_bar_id.id).await?.clone();

        // Tick up loading bar
        progress_bar.current += increment_frac;

        let display_frac = progress_bar.current / progress_bar.total;

        if f64::abs(display_frac - progress_bar.last_sent) > 0.005 {
            let message = message.unwrap_or(progress_bar.message.as_ref()).to_string();
            self.emit_progress_inner(&mut progress_bar, message, self.clock.now())
                .await?;
        }

        self.progress_storage
            .upsert(progress_bar_id.id, progress_bar)
            .await?;

        Ok(())
    }

    async fn edit_progress(
        &self,
        progress_bar_id: &ProgressBarId,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<(), EventError> {
        self.progress_storage
            .upsert_with(progress_bar_id, |progress_bar| {
                progress_bar.progress_type = event_type;
                progress_bar.total = total;
                progress_bar.message = message;
                progress_bar.current = 0.0;
                progress_bar.last_sent = 0.0;
                // A bar put to other work starts its transfer over too.
                progress_bar.bytes_done = None;
                progress_bar.bytes_total = None;
                progress_bar.speed = None;
                progress_bar.transfer = TransferWindow::default();

                Ok(())
            })
            .await?;

        self.emit_progress(progress_bar_id, 0.0, None).await?;

        Ok(())
    }

    async fn set_progress_stage(
        &self,
        progress_bar_id: &ProgressBarId,
        stage: Option<ProgressStage>,
    ) -> Result<(), EventError> {
        let mut progress_bar = self.progress_storage.get(progress_bar_id.id).await?.clone();
        progress_bar.stage = stage;

        // A new stage is shown at once, however little the fraction has moved.
        let message = progress_bar.message.clone();
        self.emit_progress_inner(&mut progress_bar, message, self.clock.now())
            .await?;

        self.progress_storage
            .upsert(progress_bar_id.id, progress_bar)
            .await?;

        Ok(())
    }

    async fn add_progress_bytes(
        &self,
        progress_bar_id: &ProgressBarId,
        bytes: u64,
    ) -> Result<(), EventError> {
        if bytes == 0 {
            return Ok(());
        }
        let now = self.clock.now();
        let mut progress_bar = self.progress_storage.get(progress_bar_id.id).await?.clone();

        let before = progress_bar.bytes_done.unwrap_or(0);
        let done = before.saturating_add(bytes);
        progress_bar.bytes_done = Some(done);
        progress_bar.transfer.record(now, done);

        let first = before == 0;
        let completes = progress_bar
            .bytes_total
            .is_some_and(|total| before < total && done >= total);
        let due = progress_bar
            .last_sent_at
            .is_none_or(|sent_at| now.saturating_duration_since(sent_at) >= BYTES_EVENT_INTERVAL);

        if first || completes || due {
            let message = progress_bar.message.clone();
            self.emit_progress_inner(&mut progress_bar, message, now)
                .await?;
        } else {
            // Kept fresh for the snapshot of the bars between events.
            progress_bar.refresh_speed(now);
        }

        self.progress_storage
            .upsert(progress_bar_id.id, progress_bar)
            .await?;

        Ok(())
    }

    async fn add_progress_bytes_total(
        &self,
        progress_bar_id: &ProgressBarId,
        bytes: u64,
    ) -> Result<(), EventError> {
        self.progress_storage
            .upsert_with(progress_bar_id, |progress_bar| {
                progress_bar.bytes_total =
                    Some(progress_bar.bytes_total.unwrap_or(0).saturating_add(bytes));
                progress_bar.bytes_done.get_or_insert(0);

                Ok(())
            })
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::mpsc;

    use crate::features::events::{
        Event, EventEmitter, ListProgressBarsUseCase, ListProgressBarsUseCasePort, ProgressEvent,
        infra::InMemoryProgressBarStorage,
    };

    use super::*;

    struct ChannelEmitter(mpsc::UnboundedSender<Event>);

    #[async_trait]
    impl EventEmitter<Event> for ChannelEmitter {
        async fn emit(&self, event: Event) -> Result<(), EventError> {
            let _ = self.0.send(event);
            Ok(())
        }

        fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
    }

    fn service() -> (
        ProgressServiceImpl<InMemoryProgressBarStorage>,
        Arc<InMemoryProgressBarStorage>,
        mpsc::UnboundedReceiver<Event>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        let storage = Arc::new(InMemoryProgressBarStorage::default());
        let service = ProgressServiceImpl::new(Arc::new(ChannelEmitter(tx)), storage.clone());
        (service, storage, rx)
    }

    /// A clock that moves only when told to.
    struct ManualClock(std::sync::Mutex<Instant>);

    impl ManualClock {
        fn advance(&self, by: Duration) {
            *self.0.lock().unwrap() += by;
        }
    }

    impl Clock for ManualClock {
        fn now(&self) -> Instant {
            *self.0.lock().unwrap()
        }
    }

    fn clocked_service() -> (
        ProgressServiceImpl<InMemoryProgressBarStorage>,
        Arc<InMemoryProgressBarStorage>,
        mpsc::UnboundedReceiver<Event>,
        Arc<ManualClock>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        let storage = Arc::new(InMemoryProgressBarStorage::default());
        let clock = Arc::new(ManualClock(std::sync::Mutex::new(Instant::now())));
        let service = ProgressServiceImpl::with_clock(
            Arc::new(ChannelEmitter(tx)),
            storage.clone(),
            clock.clone(),
        );
        (service, storage, rx, clock)
    }

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    async fn next_progress(rx: &mut mpsc::UnboundedReceiver<Event>) -> ProgressEvent {
        let Some(Event::Progress(event)) = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .unwrap()
        else {
            panic!("expected a progress event");
        };
        event
    }

    fn java() -> ProgressEventType {
        ProgressEventType::JavaDownload { version: 21 }
    }

    #[tokio::test]
    async fn events_of_a_new_bar_are_running_with_no_extra_detail() {
        let (service, storage, mut rx) = service();

        let id = service
            .init_progress(java(), 10.0, "Downloading".to_string())
            .await
            .unwrap();
        service.emit_progress(&id, 2.0, None).await.unwrap();
        service.emit_progress(&id, 3.0, Some("Half")).await.unwrap();

        let fifth = next_progress(&mut rx).await;
        let half = next_progress(&mut rx).await;
        assert_eq!(fifth.fraction, Some(0.2));
        assert_eq!(fifth.message, "Downloading");
        assert_eq!(half.fraction, Some(0.5));
        assert_eq!(half.message, "Half");
        for event in [&fifth, &half] {
            assert_eq!(event.progress_bar_id, id.id);
            assert_eq!(event.status, ProgressStatus::Running);
            assert_eq!(event.stage, None);
            assert_eq!(event.parent_id, None);
            assert_eq!(
                (event.bytes_done, event.bytes_total, event.speed),
                (None, None, None)
            );
        }

        let bar = storage.get(id.id).await.unwrap().clone();
        assert_eq!(bar.status, ProgressStatus::Running);
        assert_eq!(bar.stage, None);
        assert_eq!(bar.parent_id, None);
        assert_eq!(
            (bar.bytes_done, bar.bytes_total, bar.speed),
            (None, None, None)
        );
    }

    #[tokio::test]
    async fn stage_is_sent_at_once_and_kept_for_later_events() {
        let (service, storage, mut rx) = service();
        let id = service
            .init_progress(java(), 10.0, "Downloading".to_string())
            .await
            .unwrap();

        service
            .set_progress_stage(&id, Some(ProgressStage::Verifying))
            .await
            .unwrap();
        let staged = next_progress(&mut rx).await;
        assert_eq!(staged.stage, Some(ProgressStage::Verifying));
        assert_eq!(staged.fraction, Some(0.0));
        assert_eq!(staged.message, "Downloading");
        assert_eq!(staged.status, ProgressStatus::Running);

        service.emit_progress(&id, 5.0, None).await.unwrap();
        assert_eq!(
            next_progress(&mut rx).await.stage,
            Some(ProgressStage::Verifying)
        );
        assert_eq!(
            storage.get(id.id).await.unwrap().stage,
            Some(ProgressStage::Verifying)
        );
    }

    #[tokio::test]
    async fn bar_ends_done_when_dropped_and_failed_after_fail() {
        let (service, _storage, mut rx) = service();

        let ok = service
            .init_progress(java(), 1.0, "Downloading".to_string())
            .await
            .unwrap();
        drop(ok);
        let done = next_progress(&mut rx).await;
        assert_eq!(done.fraction, None);
        assert_eq!(done.message, "Completed");
        assert_eq!(done.status, ProgressStatus::Done);

        let broken = service
            .init_progress(java(), 1.0, "Downloading".to_string())
            .await
            .unwrap();
        broken.fail("No space left").await;
        drop(broken);
        let failed = next_progress(&mut rx).await;
        assert_eq!(failed.fraction, None);
        assert_eq!(failed.message, "No space left");
        assert_eq!(
            failed.status,
            ProgressStatus::Failed {
                message: "No space left".to_string()
            }
        );
    }

    /// Every event waiting in `rx`.
    fn sent(rx: &mut mpsc::UnboundedReceiver<Event>) -> Vec<ProgressEvent> {
        std::iter::from_fn(|| rx.try_recv().ok())
            .map(|event| {
                let Event::Progress(event) = event else {
                    panic!("expected a progress event");
                };
                event
            })
            .collect()
    }

    async fn snapshot(storage: &Arc<InMemoryProgressBarStorage>) -> ProgressBar {
        let bars = ListProgressBarsUseCase::new(storage.clone())
            .execute()
            .await;
        assert_eq!(bars.len(), 1);
        bars.into_iter().next().unwrap()
    }

    #[tokio::test]
    async fn bytes_and_speed_are_in_events_and_in_the_snapshot() {
        let (service, storage, mut rx, clock) = clocked_service();
        let id = service
            .init_progress(java(), 10.0, "Downloading".to_string())
            .await
            .unwrap();

        service.add_progress_bytes_total(&id, 10_000).await.unwrap();
        assert!(sent(&mut rx).is_empty(), "the total alone sends nothing");
        let bar = snapshot(&storage).await;
        assert_eq!((bar.bytes_done, bar.bytes_total), (Some(0), Some(10_000)));

        service.add_progress_bytes(&id, 1000).await.unwrap();
        let first = next_progress(&mut rx).await;
        assert_eq!(first.fraction, Some(0.0));
        assert_eq!(first.message, "Downloading");
        assert_eq!(
            (first.bytes_done, first.bytes_total, first.speed),
            (Some(1000), Some(10_000), None)
        );

        clock.advance(ms(500));
        service.add_progress_bytes(&id, 1000).await.unwrap();
        let second = next_progress(&mut rx).await;
        assert_eq!(
            (second.bytes_done, second.speed),
            (Some(2000), Some(2000.0))
        );

        // Two seconds on, the first report has left the window.
        clock.advance(ms(2000));
        service.add_progress_bytes(&id, 1000).await.unwrap();
        let third = next_progress(&mut rx).await;
        assert_eq!((third.bytes_done, third.speed), (Some(3000), Some(500.0)));

        let bar = snapshot(&storage).await;
        assert_eq!(
            (bar.bytes_done, bar.bytes_total, bar.speed),
            (Some(3000), Some(10_000), Some(500.0))
        );
    }

    #[tokio::test]
    async fn byte_events_are_spaced_but_the_first_and_the_completing_ones_go_out() {
        let (service, storage, mut rx, clock) = clocked_service();
        let id = service
            .init_progress(java(), 10.0, "Downloading".to_string())
            .await
            .unwrap();
        service.add_progress_bytes_total(&id, 1000).await.unwrap();

        service.add_progress_bytes(&id, 100).await.unwrap();
        assert_eq!(sent(&mut rx).len(), 1, "the first bytes go out at once");

        clock.advance(ms(100));
        service.add_progress_bytes(&id, 100).await.unwrap();
        clock.advance(ms(100));
        service.add_progress_bytes(&id, 100).await.unwrap();
        assert!(sent(&mut rx).is_empty(), "within 250 ms of the last event");
        assert_eq!(snapshot(&storage).await.bytes_done, Some(300));

        clock.advance(ms(60));
        service.add_progress_bytes(&id, 100).await.unwrap();
        let due = sent(&mut rx);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].bytes_done, Some(400));

        clock.advance(ms(10));
        service.add_progress_bytes(&id, 600).await.unwrap();
        let last = sent(&mut rx);
        assert_eq!(
            last.len(),
            1,
            "the bytes completing the total go out at once"
        );
        assert_eq!(
            (last[0].bytes_done, last[0].bytes_total, last[0].speed),
            (Some(1000), Some(1000), None)
        );

        clock.advance(ms(10));
        service.add_progress_bytes(&id, 50).await.unwrap();
        assert!(sent(&mut rx).is_empty());
    }

    #[tokio::test]
    async fn byte_events_are_spaced_from_fraction_events_too() {
        let (service, _storage, mut rx, clock) = clocked_service();
        let id = service
            .init_progress(java(), 10.0, "Downloading".to_string())
            .await
            .unwrap();
        service.add_progress_bytes(&id, 100).await.unwrap();
        clock.advance(ms(300));

        service.emit_progress(&id, 1.0, None).await.unwrap();
        clock.advance(ms(100));
        service.add_progress_bytes(&id, 100).await.unwrap();

        let events = sent(&mut rx);
        assert_eq!(events.len(), 2, "{events:?}");
        assert_eq!(events[1].fraction, Some(0.1));
        assert_eq!(events[1].bytes_done, Some(100));
    }

    #[tokio::test]
    async fn editing_a_bar_starts_its_transfer_over() {
        let (service, storage, _rx, _clock) = clocked_service();
        let id = service
            .init_progress(java(), 10.0, "Downloading".to_string())
            .await
            .unwrap();
        service.add_progress_bytes_total(&id, 1000).await.unwrap();
        service.add_progress_bytes(&id, 100).await.unwrap();

        service
            .edit_progress(&id, java(), 5.0, "Extracting".to_string())
            .await
            .unwrap();

        let bar = snapshot(&storage).await;
        assert_eq!(
            (bar.bytes_done, bar.bytes_total, bar.speed),
            (None, None, None)
        );
    }
}
