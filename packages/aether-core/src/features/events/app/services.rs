use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::MutexGuard;
use uuid::Uuid;

use crate::features::events::{
    EventError, ProgressBar, ProgressBarId, ProgressBarStorage, ProgressBarStorageError,
    ProgressEvent, ProgressEventType, ProgressStage, ProgressStatus, SharedEventEmitter,
    SharedProgressWait,
    app::ports::ProgressService,
    domain::{Shown, TransferWindow},
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

/// A bar waiting on a shared download ([`ProgressService::wait_on_shared`]).
struct Wait {
    token: u64,
    event_type: ProgressEventType,
    waiting: Uuid,
}

#[derive(Default)]
struct Waits {
    next_token: u64,
    entries: Vec<Wait>,
}

/// A bar due to be sent, taken in the change that made it so rather than copied whole: its
/// event, and for a shared bar what its mirrors are to show and which they are.
struct Due {
    event: ProgressEvent,
    mirrored: Option<(Shown, Vec<Uuid>)>,
}

impl Due {
    /// `bar` as it is now, with `message` or, without one, its own.
    fn of(bar: &ProgressBar, message: Option<&str>) -> Self {
        let message = message.map_or_else(|| bar.message.clone(), ToOwned::to_owned);
        Self {
            event: bar.running_event(message),
            mirrored: (!bar.mirrors.is_empty()).then(|| (bar.shown(), bar.mirrors.clone())),
        }
    }
}

/// Bars form a tree through [`init_child_progress`](ProgressService::init_child_progress). A
/// parent counts the bytes of its children as its own: every byte, and every byte expected, a
/// bar is told of is added to each bar above it too, so a parent's bytes and speed are those of
/// all its children together, the ones already done included. A parent ends no later than its
/// children ([`ProgressBarId`] takes them with it).
///
/// A shared bar ([`init_shared_progress`](ProgressService::init_shared_progress)) stands at the
/// top, and each bar waiting on it has a mirror of it among its children. Every event of the
/// shared bar goes to its mirrors too, as their own; a mirror is never told of bytes itself, so
/// they never reach its parent.
pub struct ProgressServiceImpl<PS: ProgressBarStorage + 'static> {
    pub event_emitter: SharedEventEmitter,
    progress_storage: Arc<PS>,
    clock: Arc<dyn Clock>,
    /// Keeps the changes that span bars (the bytes of a child reaching its parents, a shared
    /// bar's event reaching its mirrors, a mirror joining its shared bar) whole and in order.
    /// Each change to one bar is atomic in the storage by itself. Never held while events are
    /// sent ([`emit_after`](Self::emit_after)).
    update_lock: tokio::sync::Mutex<()>,
    /// Sends the events of the changes in the order the changes were made: taken before the
    /// update lock is let go, so no event of a bar overtakes an earlier one of it.
    emit_lock: tokio::sync::Mutex<()>,
    waits: Arc<Mutex<Waits>>,
    /// The shared bars started, some of them gone by now: where a bar starting to wait looks
    /// for one running. Changed under the update lock.
    shared_bars: Mutex<Vec<Uuid>>,
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
            emit_lock: tokio::sync::Mutex::new(()),
            waits: Arc::default(),
            shared_bars: Mutex::default(),
        }
    }

    /// Send `events`, made under the update lock `guard`, after those of every change made
    /// before them, without holding up the changes made after them.
    async fn emit_after(
        &self,
        guard: MutexGuard<'_, ()>,
        events: Vec<ProgressEvent>,
    ) -> Result<(), EventError> {
        if events.is_empty() {
            return Ok(());
        }
        let _emitting = self.emit_lock.lock().await;
        drop(guard);
        for event in events {
            self.event_emitter.emit(event.into()).await?;
        }
        Ok(())
    }

    /// Change the bar `id` in place with `change`, and hand back what it returns: `NotFound`
    /// for a bar that is gone.
    async fn change<R: Send>(
        &self,
        id: Uuid,
        change: impl FnOnce(&mut ProgressBar) -> R + Send,
    ) -> Result<R, ProgressBarStorageError> {
        let mut result = None;
        self.progress_storage
            .update(id, Box::new(|bar| result = Some(change(bar))))
            .await?;
        Ok(result.expect("the storage applies the change to a bar it has"))
    }

    /// Register a bar, under `parent_id` when there is one. A shared bar gets a mirror under
    /// every bar waiting on its `event_type`.
    async fn start(
        &self,
        parent_id: Option<Uuid>,
        event_type: ProgressEventType,
        total: f64,
        message: String,
        shared: bool,
    ) -> Result<ProgressBarId, EventError> {
        let guard = self.update_lock.lock().await;

        if let Some(parent_id) = parent_id {
            // Fails for a parent that is gone: a child of nothing would never be taken away.
            self.change(parent_id, |parent| parent.has_children = true)
                .await?;
        }

        let id = Uuid::new_v4();
        let mut bar = ProgressBar::new(id, event_type, total, message, parent_id);
        bar.shared = shared;
        let waiting = shared.then(|| self.waiting_on(&bar.progress_type));
        self.progress_storage.insert(id, bar).await?;
        if shared {
            self.shared_bars
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(id);
        }

        let mut events = Vec::new();
        for waiting in waiting.into_iter().flatten() {
            events.extend(self.attach_mirror(id, waiting).await?);
        }
        self.emit_after(guard, events).await?;

        Ok(ProgressBarId {
            id,
            progress_storage: Some(self.progress_storage.clone()),
            event_emitter: Some(self.event_emitter.clone()),
        })
    }

    /// The bars waiting on the shared download `event_type`, each once.
    fn waiting_on(&self, event_type: &ProgressEventType) -> Vec<Uuid> {
        let waits = self.waits.lock().unwrap_or_else(PoisonError::into_inner);
        let mut waiting: Vec<_> = waits
            .entries
            .iter()
            .filter(|wait| wait.event_type == *event_type)
            .map(|wait| wait.waiting)
            .collect();
        waiting.sort_unstable();
        waiting.dedup();
        waiting
    }

    /// The running shared bar of `event_type`, if any. The shared bars that are gone are
    /// forgotten. Called under the update lock.
    async fn running_shared(&self, event_type: &ProgressEventType) -> Option<Uuid> {
        let known = self
            .shared_bars
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let mut running = None;
        let mut gone = Vec::new();
        for id in known {
            match self.progress_storage.get(id).await {
                Ok(bar) => {
                    if running.is_none()
                        && bar.status == ProgressStatus::Running
                        && bar.progress_type == *event_type
                    {
                        running = Some(id);
                    }
                }
                Err(_) => gone.push(id),
            }
        }
        if !gone.is_empty() {
            self.shared_bars
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .retain(|id| !gone.contains(id));
        }
        running
    }

    /// Put a mirror of the shared bar `shared_id` under the bar `parent_id`, unless one is
    /// there already; nothing when either is gone. Hands back the mirror's first event, to
    /// send, when the shared bar has been shown already. Called under the update lock.
    async fn attach_mirror(
        &self,
        shared_id: Uuid,
        parent_id: Uuid,
    ) -> Result<Option<ProgressEvent>, EventError> {
        let Ok(known) = self
            .progress_storage
            .get(shared_id)
            .await
            .map(|shared| shared.mirrors.clone())
        else {
            return Ok(None);
        };
        for mirror_id in known {
            let mirrored = self
                .progress_storage
                .get(mirror_id)
                .await
                .is_ok_and(|mirror| mirror.parent_id == Some(parent_id));
            if mirrored {
                return Ok(None);
            }
        }
        if self
            .change(parent_id, |parent| parent.has_children = true)
            .await
            .is_err()
        {
            return Ok(None);
        }

        // From here on the end of the shared bar takes the mirror along.
        let mirror_id = Uuid::new_v4();
        let Ok((mirror, shown)) = self
            .change(shared_id, |shared| {
                shared.mirrors.push(mirror_id);
                (
                    ProgressBar::mirror(mirror_id, shared, parent_id),
                    shared.last_sent_at.is_some(),
                )
            })
            .await
        else {
            return Ok(None);
        };
        // Shown at once when the shared bar has been, so a bar starting to wait on a running
        // download is not left blank until its next event.
        let event = shown.then(|| mirror.running_event(mirror.message.clone()));
        self.progress_storage.insert(mirror_id, mirror).await?;

        // A shared bar that ended before the mirror was in passed over it.
        if self.progress_storage.get(shared_id).await.is_err() {
            self.progress_storage.remove(mirror_id).await?;
            return Ok(None);
        }
        Ok(event)
    }

    /// [`emit_progress`](ProgressService::emit_progress), under the update lock: the events to
    /// send go to `events`.
    async fn advance(
        &self,
        progress_bar_id: &ProgressBarId,
        increment_frac: f64,
        message: Option<&str>,
        events: &mut Vec<ProgressEvent>,
    ) -> Result<(), EventError> {
        let now = self.clock.now();
        let due = self
            .change(progress_bar_id.id, |bar| {
                // Tick up loading bar
                bar.current += increment_frac;
                let display_frac = bar.current / bar.total;

                (f64::abs(display_frac - bar.last_sent) > 0.005).then(|| {
                    mark_sent(bar, now);
                    Due::of(bar, message)
                })
            })
            .await?;

        if let Some(due) = due {
            self.collect(due, events).await;
        }
        Ok(())
    }

    /// Put the events of a bar `due` to be sent into `events`: those of its mirrors first, each
    /// made to show it, then its own. Called under the update lock.
    async fn collect(&self, due: Due, events: &mut Vec<ProgressEvent>) {
        if let Some((shown, mirrors)) = &due.mirrored {
            for &mirror_id in mirrors {
                // A mirror whose parent ended is gone; the shared bar's end passes over it.
                let mirrored = self
                    .change(mirror_id, |mirror| {
                        mirror.show(shown);
                        mirror.running_event(due.event.message.clone())
                    })
                    .await;
                if let Ok(event) = mirrored {
                    events.push(event);
                }
            }
        }
        events.push(due.event);
    }
}

/// Note that `bar` is sent as it is at `now`.
fn mark_sent(bar: &mut ProgressBar, now: Instant) {
    bar.refresh_speed(now);
    bar.last_sent = bar.current / bar.total;
    bar.last_sent_at = Some(now);
}

/// Count `bytes` more as transferred for `bar` at `now`: the bar as it is to be sent, when it is
/// time to tell listeners.
fn record_bytes(bar: &mut ProgressBar, bytes: u64, now: Instant) -> Option<Due> {
    let before = bar.bytes_done.unwrap_or(0);
    let done = before.saturating_add(bytes);
    bar.bytes_done = Some(done);
    bar.transfer.record(now, done);

    let first = before == 0;
    let completes = bar
        .bytes_total
        .is_some_and(|total| before < total && done >= total);
    let due = bar
        .last_sent_at
        .is_none_or(|sent_at| now.saturating_duration_since(sent_at) >= BYTES_EVENT_INTERVAL);

    if first || completes || due {
        mark_sent(bar, now);
        Some(Due::of(bar, None))
    } else {
        // Kept fresh for the snapshot of the bars between events.
        bar.refresh_speed(now);
        None
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
        self.start(None, event_type, total, message, false).await
    }

    async fn init_child_progress(
        &self,
        parent_id: Uuid,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        self.start(Some(parent_id), event_type, total, message, false)
            .await
    }

    async fn init_shared_progress(
        &self,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        self.start(None, event_type, total, message, true).await
    }

    async fn wait_on_shared(
        &self,
        waiting: Option<&ProgressBarId>,
        event_type: ProgressEventType,
    ) -> SharedProgressWait {
        let Some(waiting) = waiting else {
            return SharedProgressWait::none();
        };
        let guard = self.update_lock.lock().await;

        let running = self.running_shared(&event_type).await;

        let token = {
            let mut waits = self.waits.lock().unwrap_or_else(PoisonError::into_inner);
            let token = waits.next_token;
            waits.next_token += 1;
            waits.entries.push(Wait {
                token,
                event_type,
                waiting: waiting.id,
            });
            token
        };

        let mut events = Vec::new();
        if let Some(shared_id) = running {
            match self.attach_mirror(shared_id, waiting.id).await {
                Ok(event) => events.extend(event),
                Err(error) => log::debug!("Failed to send progress: {error}"),
            }
        }
        if let Err(error) = self.emit_after(guard, events).await {
            log::debug!("Failed to send progress: {error}");
        }

        let waits = self.waits.clone();
        SharedProgressWait::new(move || {
            waits
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .entries
                .retain(|wait| wait.token != token);
        })
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
        let guard = self.update_lock.lock().await;
        let mut events = Vec::new();
        self.advance(progress_bar_id, increment_frac, message, &mut events)
            .await?;
        self.emit_after(guard, events).await
    }

    async fn edit_progress(
        &self,
        progress_bar_id: &ProgressBarId,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<(), EventError> {
        let guard = self.update_lock.lock().await;
        // The bytes it had stay counted in its parent, if any: they were transferred.
        self.change(progress_bar_id.id, |progress_bar| {
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
        })
        .await?;

        let mut events = Vec::new();
        self.advance(progress_bar_id, 0.0, None, &mut events)
            .await?;
        self.emit_after(guard, events).await
    }

    async fn set_progress_stage(
        &self,
        progress_bar_id: &ProgressBarId,
        stage: Option<ProgressStage>,
    ) -> Result<(), EventError> {
        let guard = self.update_lock.lock().await;
        let now = self.clock.now();
        // A new stage is shown at once, however little the fraction has moved.
        let due = self
            .change(progress_bar_id.id, |bar| {
                bar.stage = stage;
                mark_sent(bar, now);
                Due::of(bar, None)
            })
            .await?;

        let mut events = Vec::new();
        self.collect(due, &mut events).await;
        self.emit_after(guard, events).await
    }

    async fn add_progress_bytes(
        &self,
        progress_bar_id: &ProgressBarId,
        bytes: u64,
    ) -> Result<(), EventError> {
        if bytes == 0 {
            return Ok(());
        }
        let guard = self.update_lock.lock().await;
        let now = self.clock.now();

        let own = progress_bar_id.id;
        let mut next = Some(own);
        let mut events = Vec::new();
        while let Some(bar_id) = next {
            let changed = self
                .change(bar_id, |bar| (bar.parent_id, record_bytes(bar, bytes, now)))
                .await;
            let (parent_id, due) = match changed {
                Ok(changed) => changed,
                // A parent that is gone already took its children along, so only an update
                // racing its end gets here.
                Err(_) if bar_id != own => break,
                Err(err) => return Err(err.into()),
            };
            next = parent_id;

            if let Some(due) = due {
                self.collect(due, &mut events).await;
            }
        }

        self.emit_after(guard, events).await
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
            let changed = self
                .change(bar_id, |bar| {
                    bar.bytes_total = Some(bar.bytes_total.unwrap_or(0).saturating_add(bytes));
                    bar.bytes_done.get_or_insert(0);
                    bar.parent_id
                })
                .await;
            next = match changed {
                Ok(parent_id) => parent_id,
                Err(_) if bar_id != own => break,
                Err(err) => return Err(err.into()),
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::mpsc;

    use crate::features::events::{
        Event, EventEmitter, ListProgressBarsUseCase, ListProgressBarsUseCasePort,
        infra::InMemoryProgressBarStorage,
        test_support::{ChannelEmitter, mirrors_now as mirrors, pack, sent},
    };

    use super::*;

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
    async fn a_bar_started_under_a_parent_or_none_goes_where_it_is_told() {
        use crate::features::events::ProgressServiceExt;

        let (service, storage, _rx) = service();
        let parent = service
            .init_progress(java(), 1.0, String::new())
            .await
            .unwrap();

        let under = service
            .init_progress_under_safe(Some(&parent), java(), 1.0, String::new())
            .await
            .unwrap();
        let alone = service
            .init_progress_under_safe(None, java(), 1.0, String::new())
            .await
            .unwrap();

        assert_eq!(bar(&storage, &under).await.parent_id, Some(parent.id));
        assert_eq!(bar(&storage, &alone).await.parent_id, None);
    }

    #[tokio::test]
    async fn a_shared_bar_is_mirrored_under_every_bar_waiting_on_it() {
        let (service, storage, _rx) = service();
        let a = service
            .init_progress(pack("a"), 1.0, String::new())
            .await
            .unwrap();
        let b = service
            .init_progress(pack("b"), 1.0, String::new())
            .await
            .unwrap();
        let unrelated = service
            .init_progress(pack("c"), 1.0, String::new())
            .await
            .unwrap();

        // `a` waits before the download starts, `b` once it runs.
        let _a_waits = service.wait_on_shared(Some(&a), java()).await;
        let _other = service
            .wait_on_shared(
                Some(&unrelated),
                ProgressEventType::JavaDownload { version: 8 },
            )
            .await;
        assert!(mirrors(&storage).await.is_empty(), "nothing to wait on yet");
        let shared = service
            .init_shared_progress(java(), 1.0, String::new())
            .await
            .unwrap();
        let _b_waits = service.wait_on_shared(Some(&b), java()).await;

        let mut expected = vec![(a.id, shared.id), (b.id, shared.id)];
        expected.sort_unstable();
        assert_eq!(mirrors(&storage).await, expected);
        assert_eq!(bar(&storage, &shared).await.parent_id, None);
        // Waiting again adds no second mirror.
        let _again = service.wait_on_shared(Some(&a), java()).await;
        assert_eq!(mirrors(&storage).await.len(), 2);
    }

    #[tokio::test]
    async fn a_wait_that_ended_gets_no_mirror() {
        let (service, storage, _rx) = service();
        let pack_bar = service
            .init_progress(pack("a"), 1.0, String::new())
            .await
            .unwrap();

        drop(service.wait_on_shared(Some(&pack_bar), java()).await);
        let _shared = service
            .init_shared_progress(java(), 1.0, String::new())
            .await
            .unwrap();

        assert!(mirrors(&storage).await.is_empty());
    }

    #[tokio::test]
    async fn a_mirror_shows_the_shared_bar_and_its_bytes_stay_out_of_its_parent() {
        let (service, storage, mut rx, clock) = clocked_service();
        let pack_bar = service
            .init_progress(pack("a"), 1.0, String::new())
            .await
            .unwrap();
        let file_bar = service
            .init_child_progress(pack_bar.id, file("a.jar"), 1.0, String::new())
            .await
            .unwrap();
        service
            .add_progress_bytes_total(&file_bar, 10)
            .await
            .unwrap();
        service.add_progress_bytes(&file_bar, 4).await.unwrap();
        let _waits = service.wait_on_shared(Some(&pack_bar), java()).await;
        let shared = service
            .init_shared_progress(java(), 10.0, "Downloading".to_string())
            .await
            .unwrap();
        let library = service
            .init_child_progress(shared.id, file("lib.jar"), 1.0, String::new())
            .await
            .unwrap();
        sent(&mut rx);

        service
            .add_progress_bytes_total(&library, 1000)
            .await
            .unwrap();
        service.add_progress_bytes(&library, 100).await.unwrap();
        clock.advance(ms(500));
        service.add_progress_bytes(&library, 300).await.unwrap();
        service.emit_progress(&shared, 5.0, None).await.unwrap();
        service
            .set_progress_stage(&shared, Some(ProgressStage::Extracting))
            .await
            .unwrap();

        let events = sent(&mut rx);
        let shared_events: Vec<_> = events
            .iter()
            .filter(|event| event.progress_bar_id == shared.id)
            .collect();
        let mirror_events: Vec<_> = events
            .iter()
            .filter(|event| event.mirror_of == Some(shared.id))
            .collect();
        assert_eq!(mirror_events.len(), shared_events.len(), "{events:?}");
        for (mirror, shared_event) in mirror_events.iter().zip(&shared_events) {
            assert_eq!(mirror.parent_id, Some(pack_bar.id));
            assert_eq!(mirror.event, java());
            assert_eq!(
                (
                    mirror.fraction,
                    mirror.bytes_done,
                    mirror.bytes_total,
                    mirror.speed,
                    mirror.stage
                ),
                (
                    shared_event.fraction,
                    shared_event.bytes_done,
                    shared_event.bytes_total,
                    shared_event.speed,
                    shared_event.stage
                )
            );
        }
        let last = mirror_events.last().unwrap();
        assert_eq!(
            (last.fraction, last.bytes_done, last.stage),
            (Some(0.5), Some(400), Some(ProgressStage::Extracting))
        );
        assert_eq!(last.speed, Some(600.0));

        let pack_bar = bar(&storage, &pack_bar).await;
        assert_eq!(
            (pack_bar.bytes_done, pack_bar.bytes_total),
            (Some(4), Some(10)),
            "the shared bytes are not the pack's"
        );
    }

    #[tokio::test]
    async fn a_parent_that_stops_waiting_takes_its_mirror_not_the_shared_bar() {
        let (service, storage, mut rx) = service();
        let pack_bar = service
            .init_progress(pack("a"), 1.0, String::new())
            .await
            .unwrap();
        let waits = service.wait_on_shared(Some(&pack_bar), java()).await;
        let shared = service
            .init_shared_progress(java(), 10.0, String::new())
            .await
            .unwrap();
        let [(_, _)] = mirrors(&storage).await[..] else {
            panic!("one mirror expected");
        };
        sent(&mut rx);

        pack_bar.fail("A mod is missing").await;
        drop(waits);
        drop(pack_bar);
        settle().await;

        let ended: Vec<_> = sent(&mut rx)
            .into_iter()
            .map(|event| (event.mirror_of, event.status))
            .collect();
        assert_eq!(
            ended,
            [
                (Some(shared.id), ProgressStatus::Cancelled),
                (
                    None,
                    ProgressStatus::Failed {
                        message: "A mod is missing".to_string()
                    }
                ),
            ]
        );
        assert_eq!(bar(&storage, &shared).await.status, ProgressStatus::Running);

        // The shared bar goes on, its gone mirror left out.
        service.emit_progress(&shared, 5.0, None).await.unwrap();
        let events = sent(&mut rx);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].progress_bar_id, shared.id);
        drop(shared);
        settle().await;
        assert!(storage.list().await.is_empty());
    }

    #[tokio::test]
    async fn mirrors_end_with_the_status_of_their_shared_bar() {
        let (service, storage, mut rx) = service();
        let pack_bar = service
            .init_progress(pack("a"), 1.0, String::new())
            .await
            .unwrap();
        let _waits = service.wait_on_shared(Some(&pack_bar), java()).await;
        let shared = service
            .init_shared_progress(java(), 10.0, String::new())
            .await
            .unwrap();
        sent(&mut rx);

        shared.fail("Not available").await;
        drop(shared);
        settle().await;

        let failed = ProgressStatus::Failed {
            message: "Not available".to_string(),
        };
        let ended: Vec<_> = sent(&mut rx)
            .into_iter()
            .map(|event| (event.mirror_of.is_some(), event.fraction, event.status))
            .collect();
        assert_eq!(ended, [(true, None, failed.clone()), (false, None, failed)]);
        let left: Vec<_> = storage.list().await.into_iter().map(|bar| bar.id).collect();
        assert_eq!(left, [pack_bar.id], "the waiting bar goes on");
    }

    /// How a [`CancellableBar`](crate::features::events::CancellableBar) settled by `settle`
    /// ends.
    async fn cancellable_ended_as(
        settle: impl AsyncFnOnce(crate::features::events::CancellableBar),
    ) -> ProgressStatus {
        let (service, _storage, mut rx) = service();
        let bar = service
            .init_progress(java(), 1.0, String::new())
            .await
            .unwrap();

        settle(crate::features::events::CancellableBar::new(Some(bar))).await;

        loop {
            let event = next_progress(&mut rx).await;
            if event.fraction.is_none() {
                return event.status;
            }
        }
    }

    #[tokio::test]
    async fn a_cancellable_bar_ends_by_how_it_was_settled_and_cancelled_when_dropped() {
        assert_eq!(
            cancellable_ended_as(async |bar| bar.done()).await,
            ProgressStatus::Done
        );
        assert_eq!(
            cancellable_ended_as(async |bar| bar.fail("Broken").await).await,
            ProgressStatus::Failed {
                message: "Broken".to_string()
            }
        );
        assert_eq!(
            cancellable_ended_as(async |bar| drop(bar)).await,
            ProgressStatus::Cancelled
        );
    }

    #[tokio::test]
    async fn an_update_racing_the_end_of_a_bar_does_not_bring_it_back() {
        let (service, storage, _rx) = service();
        let parent = service
            .init_progress(java(), 1.0, String::new())
            .await
            .unwrap();
        let child = service
            .init_child_progress(parent.id, file("a.jar"), 1.0, String::new())
            .await
            .unwrap();
        // The parent's end, as its handle's drop does it, between the child's updates.
        storage.remove(parent.id).await.unwrap();

        service.add_progress_bytes(&child, 10).await.unwrap();
        service.add_progress_bytes_total(&child, 10).await.unwrap();

        let left: Vec<_> = storage.list().await.into_iter().map(|bar| bar.id).collect();
        assert_eq!(left, [child.id]);
        std::mem::forget(parent);
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

    /// Sends every event into a channel, each once a permit of `gate` is there for it.
    struct GatedEmitter {
        gate: Arc<tokio::sync::Semaphore>,
        tx: mpsc::UnboundedSender<Event>,
    }

    #[async_trait]
    impl EventEmitter<Event> for GatedEmitter {
        async fn emit(&self, event: Event) -> Result<(), EventError> {
            self.gate.acquire().await.unwrap().forget();
            let _ = self.tx.send(event);
            Ok(())
        }

        fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
    }

    #[tokio::test]
    async fn a_slow_listener_holds_up_no_change_and_the_events_of_a_bar_keep_their_order() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let gate = Arc::new(tokio::sync::Semaphore::new(0));
        let storage = Arc::new(InMemoryProgressBarStorage::default());
        let clock = Arc::new(ManualClock(std::sync::Mutex::new(Instant::now())));
        let service = Arc::new(ProgressServiceImpl::with_clock(
            Arc::new(GatedEmitter {
                gate: gate.clone(),
                tx,
            }),
            storage.clone(),
            clock.clone(),
        ));
        let id = Arc::new(
            service
                .init_progress(java(), 10.0, String::new())
                .await
                .unwrap(),
        );
        let report = |bytes| {
            let (service, id) = (service.clone(), id.clone());
            tokio::spawn(async move { service.add_progress_bytes(&id, bytes).await.unwrap() })
        };

        // The first bytes go out at once: their event waits on the listener.
        let first = report(100);
        settle().await;
        assert!(!first.is_finished());

        // Bytes too soon for an event of their own are counted meanwhile.
        clock.advance(ms(10));
        tokio::time::timeout(Duration::from_secs(1), service.add_progress_bytes(&id, 100))
            .await
            .expect("the change waited on the listener")
            .unwrap();
        assert_eq!(bar(&storage, &id).await.bytes_done, Some(200));

        // A later event of the bar goes out after the earlier one.
        clock.advance(ms(300));
        let later = report(100);
        settle().await;
        gate.add_permits(2);
        first.await.unwrap();
        later.await.unwrap();

        let bytes: Vec<_> = sent(&mut rx)
            .into_iter()
            .map(|event| event.bytes_done)
            .collect();
        assert_eq!(bytes, [Some(100), Some(300)]);
    }

    /// Sends every event into a channel at once, but the `held`-th one (from 0) only once
    /// `release` has a permit for it.
    struct HoldingEmitter {
        held: usize,
        emitted: std::sync::atomic::AtomicUsize,
        release: Arc<tokio::sync::Semaphore>,
        tx: mpsc::UnboundedSender<Event>,
    }

    #[async_trait]
    impl EventEmitter<Event> for HoldingEmitter {
        async fn emit(&self, event: Event) -> Result<(), EventError> {
            let index = self
                .emitted
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if index == self.held {
                self.release.acquire().await.unwrap().forget();
            }
            let _ = self.tx.send(event);
            Ok(())
        }

        fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
    }

    /// A service whose first event waits for a permit of the semaphore handed back.
    fn held_service() -> (
        Arc<ProgressServiceImpl<InMemoryProgressBarStorage>>,
        Arc<InMemoryProgressBarStorage>,
        mpsc::UnboundedReceiver<Event>,
        Arc<tokio::sync::Semaphore>,
    ) {
        let (tx, rx) = mpsc::unbounded_channel();
        let release = Arc::new(tokio::sync::Semaphore::new(0));
        let storage = Arc::new(InMemoryProgressBarStorage::default());
        let service = Arc::new(ProgressServiceImpl::new(
            Arc::new(HoldingEmitter {
                held: 0,
                emitted: std::sync::atomic::AtomicUsize::new(0),
                release: release.clone(),
                tx,
            }),
            storage.clone(),
        ));
        (service, storage, rx, release)
    }

    /// The events of `bar` in `events`, as `(fraction, status)`.
    fn events_of(events: &[ProgressEvent], bar: Uuid) -> Vec<(Option<f64>, ProgressStatus)> {
        events
            .iter()
            .filter(|event| event.progress_bar_id == bar)
            .map(|event| (event.fraction, event.status.clone()))
            .collect()
    }

    /// The bytes of a child make a running event of its parent. While that event waits its
    /// turn behind a slow listener, the parent's handle is dropped: the parent and the child
    /// left under it end. Their final events must still be the last of them.
    #[tokio::test]
    async fn a_running_event_waiting_its_turn_does_not_come_after_the_end_of_its_bar() {
        let (service, storage, mut rx, release) = held_service();
        let other = service
            .init_progress(java(), 10.0, String::new())
            .await
            .unwrap();
        let parent = service
            .init_progress(java(), 10.0, String::new())
            .await
            .unwrap();
        let parent_id = parent.id;
        let child = service
            .init_child_progress(parent.id, file("a.jar"), 10.0, String::new())
            .await
            .unwrap();
        let child_id = child.id;
        assert!(sent(&mut rx).is_empty());

        // Another bar's event holds the line: the listener is slow with it.
        let holding = {
            let service = service.clone();
            tokio::spawn(async move { service.emit_progress(&other, 1.0, None).await.unwrap() })
        };
        settle().await;
        assert!(
            !holding.is_finished(),
            "the first event waits on the listener"
        );

        // The child's first bytes: running events of the child and the parent, made and
        // waiting behind the held one.
        let reporting = {
            let service = service.clone();
            tokio::spawn(async move {
                service.add_progress_bytes(&child, 100).await.unwrap();
                child
            })
        };
        settle().await;
        assert!(
            !reporting.is_finished(),
            "the bytes' events wait their turn"
        );
        assert_eq!(bar(&storage, &parent).await.bytes_done, Some(100));

        // The parent ends meanwhile, taking the child with it.
        drop(parent);
        settle().await;
        assert!(storage.list().await.iter().all(|bar| bar.id != parent_id));

        release.add_permits(1);
        holding.await.unwrap();
        drop(reporting.await.unwrap());
        settle().await;

        let events = sent(&mut rx);
        let order: Vec<_> = events
            .iter()
            .map(|event| (event.progress_bar_id, event.status.clone()))
            .collect();
        for (id, last) in [
            (child_id, ProgressStatus::Cancelled),
            (parent_id, ProgressStatus::Done),
        ] {
            let of_bar = events_of(&events, id);
            assert_eq!(
                of_bar.last(),
                Some(&(None, last)),
                "the final event of {id} is not its last; all events: {order:?}"
            );
        }
    }

    /// The same for one bar and its own running event: a clone of its handle reports while
    /// the handle is dropped.
    #[tokio::test]
    async fn an_own_running_event_waiting_its_turn_does_not_come_after_the_end_of_the_bar() {
        let (service, _storage, mut rx, release) = held_service();
        let other = service
            .init_progress(java(), 10.0, String::new())
            .await
            .unwrap();
        let id = service
            .init_progress(java(), 10.0, String::new())
            .await
            .unwrap();
        let bar_id = id.id;

        let holding = {
            let service = service.clone();
            tokio::spawn(async move { service.emit_progress(&other, 1.0, None).await.unwrap() })
        };
        settle().await;
        assert!(!holding.is_finished());

        let reporting = {
            let (service, clone) = (service.clone(), id.clone());
            tokio::spawn(async move {
                service.emit_progress(&clone, 5.0, None).await.unwrap();
                // Dropping the clone afterwards finds the bar gone already.
                drop(clone);
            })
        };
        settle().await;
        assert!(!reporting.is_finished());

        drop(id);
        settle().await;

        release.add_permits(1);
        holding.await.unwrap();
        reporting.await.unwrap();
        settle().await;

        let events = sent(&mut rx);
        let of_bar = events_of(&events, bar_id);
        assert_eq!(
            of_bar.last(),
            Some(&(None, ProgressStatus::Done)),
            "the final event is not the last of the bar: {of_bar:?}"
        );
    }

    #[tokio::test]
    async fn a_bar_starting_to_wait_on_a_running_download_is_shown_it_at_once() {
        let (service, _storage, mut rx) = service();
        let pack_bar = service
            .init_progress(pack("a"), 1.0, String::new())
            .await
            .unwrap();
        let shared = service
            .init_shared_progress(java(), 10.0, "Downloading".to_string())
            .await
            .unwrap();
        service.emit_progress(&shared, 4.0, None).await.unwrap();
        sent(&mut rx);

        let _wait = service.wait_on_shared(Some(&pack_bar), java()).await;

        let shown: Vec<_> = sent(&mut rx)
            .into_iter()
            .map(|event| {
                (
                    event.mirror_of,
                    event.parent_id,
                    event.fraction,
                    event.message,
                )
            })
            .collect();
        assert_eq!(
            shown,
            [(
                Some(shared.id),
                Some(pack_bar.id),
                Some(0.4),
                "Downloading".to_string()
            )]
        );
    }

    #[tokio::test]
    async fn a_wait_without_a_bar_mirrors_nothing() {
        let (service, storage, _rx) = service();
        let _shared = service
            .init_shared_progress(java(), 1.0, String::new())
            .await
            .unwrap();

        let _wait = service.wait_on_shared(None, java()).await;

        assert!(mirrors(&storage).await.is_empty());
    }

    /// What one report of bytes costs, and that a change in place of the storage loses nothing
    /// even without the update lock. Run by hand:
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

        // The bare change in place of the storage, from many tasks at once without the lock.
        for tasks in [1_usize, 8, 64] {
            let storage = Arc::new(InMemoryProgressBarStorage::default());
            let id = Uuid::new_v4();
            let mut bar = ProgressBar {
                has_children: true,
                bytes_done: Some(0),
                ..ProgressBar::new(id, java(), 1.0, "Downloading".to_string(), None)
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
                        storage
                            .update(
                                id,
                                Box::new(|bar| {
                                    bar.bytes_done = bar.bytes_done.map(|done| done + 1);
                                }),
                            )
                            .await
                            .unwrap();
                        tokio::task::yield_now().await;
                    }
                })
            });
            futures::future::join_all(workers).await;
            let elapsed = started.elapsed();
            let updates = tasks * CHUNKS;
            let counted = storage.get(id).await.unwrap().bytes_done.unwrap();
            println!(
                "bare change in place, {tasks:>2} tasks: {:.2} us per update, \
                 {} of {updates} updates lost",
                elapsed.as_secs_f64() * 1e6 / updates as f64,
                updates as u64 - counted
            );
        }
    }
}
