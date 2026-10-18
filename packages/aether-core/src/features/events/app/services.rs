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

/// Bars form a tree through [`init_child_progress`](ProgressService::init_child_progress). A
/// parent counts the bytes of its children as its own: every byte, and every byte expected, a
/// bar is told of is added to each bar above it too, so a parent's bytes and speed are those of
/// all its children together, the ones already done included. A parent ends no later than its
/// children ([`ProgressBarId`] takes them with it).
pub struct ProgressServiceImpl<PS: ProgressBarStorage + 'static> {
    pub event_emitter: SharedEventEmitter,
    progress_storage: Arc<PS>,
    clock: Arc<dyn Clock>,
    /// Every change of a bar reads it from the storage and writes it back; this keeps two of them
    /// (the bytes of two children reaching their parent) from writing over each other.
    update_lock: tokio::sync::Mutex<()>,
}

impl<PS: ProgressBarStorage + 'static> ProgressServiceImpl<PS> {
    pub fn new(event_emitter: SharedEventEmitter, progress_storage: Arc<PS>) -> Self {
        Self::with_clock(event_emitter, progress_storage, Arc::new(SystemClock))
    }

    fn with_clock(
        event_emitter: SharedEventEmitter,
        progress_storage: Arc<PS>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            event_emitter,
            progress_storage,
            clock,
            update_lock: tokio::sync::Mutex::new(()),
        }
    }

    /// Register a bar, under `parent_id` when there is one, and send its first event.
    async fn start(
        &self,
        parent_id: Option<Uuid>,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        let _guard = self.update_lock.lock().await;

        if let Some(parent_id) = parent_id {
            // Fails for a parent that is gone: a child of nothing would never be taken away.
            let mut parent = self.progress_storage.get(parent_id).await?.clone();
            parent.has_children = true;
            self.progress_storage.upsert(parent_id, parent).await?;
        }

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
                    parent_id,
                    has_children: false,
                    bytes_done: None,
                    bytes_total: None,
                    speed: None,
                    transfer: TransferWindow::default(),
                    last_sent_at: None,
                },
            )
            .await?;

        self.advance(&progress_bar_id, 0.0, None).await?;

        Ok(progress_bar_id)
    }

    /// [`emit_progress`](ProgressService::emit_progress), under the update lock.
    async fn advance(
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

    /// The bar `bar_id`, on the way up the tree from the bar `own`: an error for `own` itself,
    /// `None` for a parent that is gone already (it took its children along, so only an update
    /// racing its end gets here).
    async fn bar_on_the_way_up(
        &self,
        bar_id: Uuid,
        own: Uuid,
    ) -> Result<Option<ProgressBar>, EventError> {
        match self.progress_storage.get(bar_id).await {
            Ok(progress_bar) => Ok(Some(progress_bar.clone())),
            Err(_) if bar_id != own => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /// Count `bytes` more as transferred for `progress_bar` at `now`, and tell listeners when it
    /// is time to.
    async fn record_bytes(
        &self,
        progress_bar: &mut ProgressBar,
        bytes: u64,
        now: Instant,
    ) -> Result<(), EventError> {
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
            self.emit_progress_inner(progress_bar, message, now).await?;
        } else {
            // Kept fresh for the snapshot of the bars between events.
            progress_bar.refresh_speed(now);
        }
        Ok(())
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
        self.start(None, event_type, total, message).await
    }

    async fn init_child_progress(
        &self,
        parent_id: Uuid,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        self.start(Some(parent_id), event_type, total, message)
            .await
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
        let _guard = self.update_lock.lock().await;
        self.advance(progress_bar_id, increment_frac, message).await
    }

    async fn edit_progress(
        &self,
        progress_bar_id: &ProgressBarId,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<(), EventError> {
        let _guard = self.update_lock.lock().await;
        // The bytes it had stay counted in its parent, if any: they were transferred.
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

        self.advance(progress_bar_id, 0.0, None).await?;

        Ok(())
    }

    async fn set_progress_stage(
        &self,
        progress_bar_id: &ProgressBarId,
        stage: Option<ProgressStage>,
    ) -> Result<(), EventError> {
        let _guard = self.update_lock.lock().await;
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
        let _guard = self.update_lock.lock().await;
        let now = self.clock.now();

        let own = progress_bar_id.id;
        let mut next = Some(own);
        while let Some(bar_id) = next {
            let Some(mut progress_bar) = self.bar_on_the_way_up(bar_id, own).await? else {
                break;
            };
            next = progress_bar.parent_id;

            self.record_bytes(&mut progress_bar, bytes, now).await?;
            self.progress_storage.upsert(bar_id, progress_bar).await?;
        }

        Ok(())
    }

    async fn add_progress_bytes_total(
        &self,
        progress_bar_id: &ProgressBarId,
        bytes: u64,
    ) -> Result<(), EventError> {
        let _guard = self.update_lock.lock().await;

        let own = progress_bar_id.id;
        let mut next = Some(own);
        while let Some(bar_id) = next {
            let Some(mut progress_bar) = self.bar_on_the_way_up(bar_id, own).await? else {
                break;
            };
            next = progress_bar.parent_id;

            progress_bar.bytes_total =
                Some(progress_bar.bytes_total.unwrap_or(0).saturating_add(bytes));
            progress_bar.bytes_done.get_or_insert(0);
            self.progress_storage.upsert(bar_id, progress_bar).await?;
        }

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

    fn file(name: &str) -> ProgressEventType {
        ProgressEventType::PackFileDownload {
            instance_path: String::new(),
            pack_name: name.to_string(),
            icon: None,
            pack_version: String::new(),
        }
    }

    async fn bar(storage: &Arc<InMemoryProgressBarStorage>, id: &ProgressBarId) -> ProgressBar {
        storage.get(id.id).await.unwrap().clone()
    }

    async fn settle() {
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }
    }

    #[tokio::test]
    async fn a_child_names_its_parent_in_its_events_and_in_the_snapshot() {
        let (service, storage, mut rx) = service();
        let parent = service
            .init_progress(java(), 1.0, "Installing".to_string())
            .await
            .unwrap();

        let child = service
            .init_child_progress(parent.id, file("a.jar"), 1.0, "Downloading".to_string())
            .await
            .unwrap();
        service.emit_progress(&child, 0.5, None).await.unwrap();

        let started = next_progress(&mut rx).await;
        assert_eq!(started.progress_bar_id, child.id);
        assert_eq!(started.parent_id, Some(parent.id));
        assert_eq!(started.event, file("a.jar"));
        assert_eq!(bar(&storage, &child).await.parent_id, Some(parent.id));
        assert_eq!(bar(&storage, &parent).await.parent_id, None);
    }

    #[tokio::test]
    async fn a_child_of_a_bar_that_is_gone_is_not_started() {
        let (service, storage, _rx) = service();

        let result = service
            .init_child_progress(Uuid::new_v4(), file("a.jar"), 1.0, String::new())
            .await;

        assert!(result.is_err());
        assert!(storage.list().await.is_empty());
    }

    #[tokio::test]
    async fn a_parent_counts_the_bytes_and_the_speed_of_its_children() {
        let (service, storage, mut rx, clock) = clocked_service();
        let parent = service
            .init_progress(java(), 1.0, "Installing".to_string())
            .await
            .unwrap();
        let a = service
            .init_child_progress(parent.id, file("a.jar"), 1.0, String::new())
            .await
            .unwrap();
        let b = service
            .init_child_progress(parent.id, file("b.jar"), 1.0, String::new())
            .await
            .unwrap();
        service.add_progress_bytes_total(&a, 8000).await.unwrap();
        service.add_progress_bytes_total(&b, 6000).await.unwrap();

        // Both at 2000 bytes a second for three seconds.
        for _ in 0..=6 {
            service.add_progress_bytes(&a, 1000).await.unwrap();
            service.add_progress_bytes(&b, 1000).await.unwrap();
            clock.advance(ms(500));
        }

        let (a, b, parent_bar) = (
            bar(&storage, &a).await,
            bar(&storage, &b).await,
            bar(&storage, &parent).await,
        );
        assert_eq!((a.bytes_done, a.bytes_total), (Some(7000), Some(8000)));
        assert_eq!((b.bytes_done, b.bytes_total), (Some(7000), Some(6000)));
        assert_eq!(
            (parent_bar.bytes_done, parent_bar.bytes_total),
            (Some(14_000), Some(14_000))
        );
        assert_eq!(a.speed, Some(2000.0));
        assert_eq!(parent_bar.speed, None, "every byte expected is in");

        // The parent's own events carry the sums too.
        let parent_events: Vec<_> = sent(&mut rx)
            .into_iter()
            .filter(|event| event.progress_bar_id == parent.id)
            .collect();
        let last = parent_events.last().unwrap();
        assert_eq!(
            (last.bytes_done, last.bytes_total),
            (Some(14_000), Some(14_000))
        );
    }

    #[tokio::test]
    async fn a_parents_speed_is_the_sum_of_its_childrens() {
        let (service, storage, _rx, clock) = clocked_service();
        let parent = service
            .init_progress(java(), 1.0, String::new())
            .await
            .unwrap();
        let a = service
            .init_child_progress(parent.id, file("a.jar"), 1.0, String::new())
            .await
            .unwrap();
        let b = service
            .init_child_progress(parent.id, file("b.jar"), 1.0, String::new())
            .await
            .unwrap();

        // 2000 and 4000 bytes a second for three seconds.
        for _ in 0..=6 {
            service.add_progress_bytes(&a, 1000).await.unwrap();
            service.add_progress_bytes(&b, 2000).await.unwrap();
            clock.advance(ms(500));
        }

        assert_eq!(bar(&storage, &a).await.speed, Some(2000.0));
        assert_eq!(bar(&storage, &b).await.speed, Some(4000.0));
        assert_eq!(bar(&storage, &parent).await.speed, Some(6000.0));
    }

    #[tokio::test]
    async fn bytes_reach_every_bar_above() {
        let (service, storage, _rx) = service();
        let pack = service
            .init_progress(java(), 1.0, String::new())
            .await
            .unwrap();
        let game = service
            .init_child_progress(pack.id, java(), 1.0, String::new())
            .await
            .unwrap();
        let library = service
            .init_child_progress(game.id, file("lib.jar"), 1.0, String::new())
            .await
            .unwrap();

        service
            .add_progress_bytes_total(&library, 100)
            .await
            .unwrap();
        service.add_progress_bytes(&library, 40).await.unwrap();

        for id in [&library, &game, &pack] {
            let bar = bar(&storage, id).await;
            assert_eq!((bar.bytes_done, bar.bytes_total), (Some(40), Some(100)));
        }
    }

    #[tokio::test]
    async fn children_end_with_their_parent_and_stay_gone() {
        let (service, storage, mut rx) = service();
        let parent = service
            .init_progress(java(), 1.0, String::new())
            .await
            .unwrap();
        let parent_id = parent.id;
        let child = service
            .init_child_progress(parent.id, file("a.jar"), 1.0, String::new())
            .await
            .unwrap();
        sent(&mut rx);

        drop(parent);
        settle().await;

        assert!(storage.list().await.is_empty());
        let finished: Vec<_> = sent(&mut rx)
            .into_iter()
            .map(|event| (event.progress_bar_id, event.fraction))
            .collect();
        assert_eq!(finished, [(child.id, None), (parent_id, None)]);

        // The child's handle outlived the parent: what it reports goes nowhere.
        assert!(service.add_progress_bytes(&child, 10).await.is_err());
        drop(child);
        settle().await;
        assert!(storage.list().await.is_empty());
        assert!(sent(&mut rx).is_empty());
    }

    #[tokio::test]
    async fn a_bar_started_in_a_scope_goes_under_its_parent() {
        use crate::features::events::{ProgressServiceExt, with_progress_parent};

        let (service, storage, _rx) = service();
        let parent = service
            .init_progress(java(), 1.0, String::new())
            .await
            .unwrap();

        let inside = with_progress_parent(
            Some(&parent),
            service.init_progress_in_scope_safe(java(), 1.0, String::new()),
        )
        .await
        .unwrap();
        let outside = service
            .init_progress_in_scope_safe(java(), 1.0, String::new())
            .await
            .unwrap();
        let no_parent = with_progress_parent(
            None,
            service.init_progress_in_scope_safe(java(), 1.0, String::new()),
        )
        .await
        .unwrap();

        assert_eq!(bar(&storage, &inside).await.parent_id, Some(parent.id));
        assert_eq!(bar(&storage, &outside).await.parent_id, None);
        assert_eq!(bar(&storage, &no_parent).await.parent_id, None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn no_bytes_are_lost_when_many_children_report_at_once() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let storage = Arc::new(InMemoryProgressBarStorage::default());
        let service = Arc::new(ProgressServiceImpl::new(
            Arc::new(ChannelEmitter(tx)),
            storage.clone(),
        ));
        let parent = service
            .init_progress(java(), 1.0, String::new())
            .await
            .unwrap();

        let children = (0..16).map(|index| {
            let service = service.clone();
            let parent_id = parent.id;
            tokio::spawn(async move {
                let child = service
                    .init_child_progress(parent_id, file(&index.to_string()), 1.0, String::new())
                    .await
                    .unwrap();
                for _ in 0..100 {
                    service.add_progress_bytes(&child, 10).await.unwrap();
                    tokio::task::yield_now().await;
                }
            })
        });
        for child in futures::future::join_all(children).await {
            child.unwrap();
        }

        assert_eq!(bar(&storage, &parent).await.bytes_done, Some(16 * 100 * 10));
    }

    /// What one report of bytes costs, and what the storage's read-modify-write loses without the
    /// update lock. Run by hand:
    /// `cargo test -p aether-core measure_bar_updates -- --ignored --nocapture`.
    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    #[ignore = "a measurement, not a test"]
    #[allow(clippy::cast_precision_loss)] // timings to print
    async fn measure_bar_updates() {
        const CHUNKS: usize = 200;

        for children in [1_usize, 8, 64, 256] {
            // Events pile up unread, as cheap as an emitter can be.
            let (tx, _rx) = mpsc::unbounded_channel();
            let storage = Arc::new(InMemoryProgressBarStorage::default());
            let service = Arc::new(ProgressServiceImpl::new(
                Arc::new(ChannelEmitter(tx)),
                storage.clone(),
            ));
            let parent = service
                .init_progress(java(), 1.0, String::new())
                .await
                .unwrap();
            let mut handles = Vec::new();
            for index in 0..children {
                handles.push(
                    service
                        .init_child_progress(
                            parent.id,
                            file(&index.to_string()),
                            1.0,
                            String::new(),
                        )
                        .await
                        .unwrap(),
                );
            }

            let started = Instant::now();
            let tasks = handles.into_iter().map(|child| {
                let service = service.clone();
                tokio::spawn(async move {
                    for _ in 0..CHUNKS {
                        service.add_progress_bytes(&child, 16 * 1024).await.unwrap();
                        tokio::task::yield_now().await;
                    }
                    child
                })
            });
            let children_left = futures::future::join_all(tasks).await;
            let elapsed = started.elapsed();
            let updates = children * CHUNKS;

            let bar = bar(&storage, &parent).await;
            assert_eq!(bar.bytes_done, Some((updates * 16 * 1024) as u64));
            println!(
                "{children:>3} children x {CHUNKS} chunks: {updates:>6} updates in {elapsed:?}, \
                 {:.2} us per update (child + parent)",
                elapsed.as_secs_f64() * 1e6 / updates as f64
            );
            drop(children_left);
        }

        // The bare read-modify-write of the storage, from many tasks at once without the lock.
        for tasks in [1_usize, 8, 64] {
            let storage = Arc::new(InMemoryProgressBarStorage::default());
            let id = Uuid::new_v4();
            let mut bar = ProgressBar {
                id,
                message: "Downloading".to_string(),
                total: 1.0,
                current: 0.0,
                last_sent: 0.0,
                progress_type: java(),
                stage: None,
                status: ProgressStatus::Running,
                parent_id: None,
                has_children: true,
                bytes_done: Some(0),
                bytes_total: None,
                speed: None,
                transfer: TransferWindow::default(),
                last_sent_at: None,
            };
            // A full window: one sample per 50 ms over two seconds.
            let start = Instant::now();
            for sample in 0..40_u64 {
                bar.transfer
                    .record(start + Duration::from_millis(sample * 50), sample);
            }
            storage.insert(id, bar).await.unwrap();

            let started = Instant::now();
            let workers = (0..tasks).map(|_| {
                let storage = storage.clone();
                tokio::spawn(async move {
                    for _ in 0..CHUNKS {
                        let mut bar = storage.get(id).await.unwrap().clone();
                        bar.bytes_done = bar.bytes_done.map(|done| done + 1);
                        storage.upsert(id, bar).await.unwrap();
                        tokio::task::yield_now().await;
                    }
                })
            });
            futures::future::join_all(workers).await;
            let elapsed = started.elapsed();
            let updates = tasks * CHUNKS;
            let counted = storage.get(id).await.unwrap().bytes_done.unwrap();
            println!(
                "bare get/clone/upsert, {tasks:>2} tasks: {:.2} us per update, \
                 {} of {updates} updates lost",
                elapsed.as_secs_f64() * 1e6 / updates as f64,
                updates as u64 - counted
            );
        }
    }
}
