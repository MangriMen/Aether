use std::fmt;
use std::sync::Arc;
use std::time::Instant;

use log::error;
use uuid::Uuid;

use crate::features::events::ProgressBarStorage;

use super::event_sequencer::EventSequencer;
use super::transfer_window::TransferWindow;
use super::{ProgressEvent, ProgressEventType, ProgressStage, ProgressStatus};

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
    /// `Failed` once [`ProgressBarId::fail`] is called, `Cancelled` once
    /// [`ProgressBarId::cancel`] is; the bar is gone when it is done.
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
        }
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

#[derive(Clone)]
pub struct ProgressBarId {
    pub id: Uuid,
    pub(crate) progress_storage: Option<Arc<dyn ProgressBarStorage>>,
    pub(crate) sequencer: Option<Arc<EventSequencer>>,
}

impl fmt::Debug for ProgressBarId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProgressBarId")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

const COMPLETED_MESSAGE: &str = "Completed";
const CANCELLED_MESSAGE: &str = "Cancelled";

impl ProgressBarId {
    /// Marks the bar as failed: when the last use of it ends (the handle is dropped), listeners
    /// get a failed event with `message` instead of a completed one.
    ///
    /// Does nothing for an id that was never attached to a storage or whose bar is already gone.
    pub async fn fail(&self, message: impl Into<String> + Send) {
        self.end_as(ProgressStatus::Failed {
            message: message.into(),
        })
        .await;
    }

    /// Marks the bar as cancelled: when the last use of it ends (the handle is dropped),
    /// listeners get a cancelled event instead of a completed one.
    ///
    /// Does nothing for an id that was never attached to a storage or whose bar is already gone.
    pub async fn cancel(&self) {
        self.end_as(ProgressStatus::Cancelled).await;
    }

    /// Note how the bar is to end; the drop of the handle tells listeners.
    async fn end_as(&self, status: ProgressStatus) {
        let Some(storage) = &self.progress_storage else {
            return;
        };
        let changed = storage
            .update(self.id, Box::new(move |bar| bar.status = status))
            .await;
        if let Err(e) = changed {
            error!("Could not mark how progress bar {} ends: {e:?}", self.id);
        }
    }

    /// Ends the bars whose handles were dropped, in the order of the drops: each is unregistered
    /// with the bars still under it and its mirrors, and listeners are told that they are
    /// finished: the children first, so that none is heard of after its parent is over.
    ///
    /// The bars are taken out of the storage as one change, and the events of their end are sent
    /// in the turn of that change: after those made before it, so none of a bar comes after
    /// its end.
    async fn finish_dropped(sequencer: Arc<EventSequencer>) {
        let change = sequencer.begin_change().await;
        let mut events = Vec::new();
        for end in sequencer.take_ends() {
            Self::finish(end.id, end.storage.as_ref(), &mut events).await;
        }
        sequencer.emit_all_after(change, events).await;
    }

    /// Unregisters the bar, the bars still under it and its mirrors; their last events go to
    /// `events`. Does nothing for a bar that is gone already.
    async fn finish(
        self_id: Uuid,
        storage: &dyn ProgressBarStorage,
        events: &mut Vec<ProgressEvent>,
    ) {
        let Some(bar) = Self::unregister(self_id, storage).await else {
            return;
        };

        if bar.has_children {
            Self::finish_children(self_id, storage, events).await;
        }
        Self::finish_mirrors(&bar, storage, events).await;
        events.push(Self::finished_event(&bar));
    }

    /// End the mirrors of the shared bar `shared` as it ends: they show its last state. Their
    /// last events go to `events`.
    async fn finish_mirrors(
        shared: &ProgressBar,
        storage: &dyn ProgressBarStorage,
        events: &mut Vec<ProgressEvent>,
    ) {
        if shared.mirrors.is_empty() {
            return;
        }
        let shown = shared.shown();
        for &mirror_id in &shared.mirrors {
            // Gone already when its own parent ended first.
            let Some(mut mirror) = Self::unregister(mirror_id, storage).await else {
                continue;
            };
            mirror.show(&shown);
            mirror.status = shared.status.clone();
            events.push(Self::finished_event(&mirror));
        }
    }

    /// Finish every bar left under `parent_id`, however deep, as cancelled: whatever they were
    /// at is left unfinished. A child normally ends before its parent; one still there when the
    /// parent ends would otherwise stay in the storage, and in the UI, for good. A mirror is
    /// one of them when its parent stops waiting before the shared bar is through; the shared
    /// bar goes on. Their last events go to `events`, the deepest first, so that none comes
    /// after the end of its own parent.
    async fn finish_children(
        parent_id: Uuid,
        storage: &dyn ProgressBarStorage,
        events: &mut Vec<ProgressEvent>,
    ) {
        let mut ended = Vec::new();
        let mut parents = vec![parent_id];
        while let Some(parent_id) = parents.pop() {
            let children = storage
                .list()
                .await
                .into_iter()
                .filter(|bar| bar.parent_id == Some(parent_id));
            for child in children {
                // Its own handle may have finished it in the meantime.
                let Some(mut child) = Self::unregister(child.id, storage).await else {
                    continue;
                };
                if child.has_children {
                    parents.push(child.id);
                }
                if child.status == ProgressStatus::Running {
                    child.status = ProgressStatus::Cancelled;
                }
                ended.push(Self::finished_event(&child));
            }
        }
        // A bar is found before the bars under it.
        events.extend(ended.into_iter().rev());
    }

    /// Remove the bar from `storage`; `None` when it is not there.
    async fn unregister(id: Uuid, storage: &dyn ProgressBarStorage) -> Option<ProgressBar> {
        match storage.remove(id).await {
            Ok(removed) => removed.map(|(_, bar)| bar),
            Err(e) => {
                error!("Could not unregister progress bar {id}: {e:?}");
                None
            }
        }
    }

    /// The last event of a bar: done or, after [`fail`](Self::fail) or
    /// [`cancel`](Self::cancel), failed or cancelled.
    fn finished_event(bar: &ProgressBar) -> ProgressEvent {
        match &bar.status {
            ProgressStatus::Failed { message } => {
                bar.event(None, message.clone(), bar.status.clone())
            }
            ProgressStatus::Cancelled => bar.event(
                None,
                CANCELLED_MESSAGE.to_string(),
                ProgressStatus::Cancelled,
            ),
            ProgressStatus::Running | ProgressStatus::Done => {
                bar.event(None, COMPLETED_MESSAGE.to_string(), ProgressStatus::Done)
            }
        }
    }
}

/// Dropping the handle ends the bar's life: it is removed from storage and a final event
/// with no fraction is sent, done or, after [`ProgressBarId::fail`], failed. The bars still under
/// it end with it.
impl Drop for ProgressBarId {
    fn drop(&mut self) {
        let (Some(storage), Some(sequencer)) =
            (self.progress_storage.take(), self.sequencer.take())
        else {
            return;
        };
        sequencer.queue_end(self.id, storage);

        tokio::spawn(Self::finish_dropped(sequencer));
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use dashmap::DashMap;
    use dashmap::mapref::one::Ref as DashMapRef;
    use tokio::sync::mpsc;

    use crate::features::events::{
        Event, ProgressBarChange, ProgressBarStorage, ProgressBarStorageError,
        test_support::ChannelEmitter,
    };

    use super::*;

    /// Minimal storage double: the domain layer must not depend on the real implementation.
    #[derive(Default)]
    struct MapStorage(DashMap<Uuid, ProgressBar>);

    #[async_trait::async_trait]
    impl ProgressBarStorage for MapStorage {
        async fn insert(
            &self,
            progress_bar_id: Uuid,
            progress_bar: ProgressBar,
        ) -> Result<(), ProgressBarStorageError> {
            self.0.insert(progress_bar_id, progress_bar);
            Ok(())
        }

        async fn list(&self) -> Vec<ProgressBar> {
            self.0.iter().map(|r| r.value().clone()).collect()
        }

        async fn get(
            &self,
            progress_bar_id: Uuid,
        ) -> Result<DashMapRef<'_, Uuid, ProgressBar>, ProgressBarStorageError> {
            self.0
                .get(&progress_bar_id)
                .ok_or(ProgressBarStorageError::NotFound { progress_bar_id })
        }

        async fn update(
            &self,
            progress_bar_id: Uuid,
            change: ProgressBarChange<'_>,
        ) -> Result<(), ProgressBarStorageError> {
            let mut bar = self
                .0
                .get_mut(&progress_bar_id)
                .ok_or(ProgressBarStorageError::NotFound { progress_bar_id })?;
            change(bar.value_mut());
            Ok(())
        }

        async fn remove(
            &self,
            progress_bar_id: Uuid,
        ) -> Result<Option<(Uuid, ProgressBar)>, ProgressBarStorageError> {
            Ok(self.0.remove(&progress_bar_id))
        }
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

    fn attached(
        id: Uuid,
        storage: &Arc<MapStorage>,
        tx: mpsc::UnboundedSender<Event>,
    ) -> ProgressBarId {
        ProgressBarId {
            id,
            progress_storage: Some(storage.clone()),
            sequencer: Some(Arc::new(EventSequencer::new(Arc::new(ChannelEmitter(tx))))),
        }
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

    async fn settle() {
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }
    }

    #[tokio::test]
    async fn dropping_progress_bar_id_removes_bar_and_emits_completion() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let storage = Arc::new(MapStorage::default());
        let id = Uuid::new_v4();
        storage.insert(id, bar(id)).await.unwrap();
        let bar_id = ProgressBarId {
            id,
            progress_storage: Some(storage.clone()),
            sequencer: Some(Arc::new(EventSequencer::new(Arc::new(ChannelEmitter(tx))))),
        };
        assert_eq!(storage.list().await.len(), 1);

        drop(bar_id);

        let Some(Event::Progress(done)) = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .unwrap()
        else {
            panic!("expected completion event");
        };
        assert_eq!(done.progress_bar_id, id);
        assert_eq!(done.fraction, None);
        assert_eq!(done.message, "Completed");
        assert_eq!(done.status, ProgressStatus::Done);
        assert_eq!(done.event, jar_event());
        assert_eq!(done.stage, None);
        assert_eq!(done.parent_id, None);
        assert_eq!(
            (done.bytes_done, done.bytes_total, done.speed),
            (None, None, None)
        );
        assert!(storage.list().await.is_empty());
    }

    #[tokio::test]
    async fn dropping_failed_progress_bar_id_emits_failure_instead_of_completion() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let storage = Arc::new(MapStorage::default());
        let id = Uuid::new_v4();
        storage.insert(id, bar(id)).await.unwrap();
        let bar_id = attached(id, &storage, tx);

        bar_id.fail("Checksum mismatch").await;
        assert!(rx.try_recv().is_err(), "fail alone must not emit");
        assert_eq!(
            storage.get(id).await.unwrap().status,
            ProgressStatus::Failed {
                message: "Checksum mismatch".to_string()
            }
        );
        drop(bar_id);

        let failed = next_progress(&mut rx).await;
        assert_eq!(failed.progress_bar_id, id);
        assert_eq!(failed.fraction, None);
        assert_eq!(failed.message, "Checksum mismatch");
        assert_eq!(
            failed.status,
            ProgressStatus::Failed {
                message: "Checksum mismatch".to_string()
            }
        );
        assert!(storage.list().await.is_empty());
    }

    #[tokio::test]
    async fn final_event_carries_the_bars_stage_parent_and_bytes() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let storage = Arc::new(MapStorage::default());
        let id = Uuid::new_v4();
        let parent = Uuid::new_v4();
        storage
            .insert(
                id,
                ProgressBar {
                    stage: Some(ProgressStage::Verifying),
                    parent_id: Some(parent),
                    bytes_done: Some(4),
                    bytes_total: Some(10),
                    speed: Some(2.5),
                    ..bar(id)
                },
            )
            .await
            .unwrap();

        drop(attached(id, &storage, tx));

        let done = next_progress(&mut rx).await;
        assert_eq!(done.stage, Some(ProgressStage::Verifying));
        assert_eq!(done.parent_id, Some(parent));
        assert_eq!(
            (done.bytes_done, done.bytes_total, done.speed),
            (Some(4), Some(10), Some(2.5))
        );
    }

    #[tokio::test]
    async fn dropping_a_parent_finishes_the_bars_left_under_it_before_itself() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let storage = Arc::new(MapStorage::default());
        let [parent, child, grandchild, unrelated] = [(); 4].map(|()| Uuid::new_v4());
        for bar in [
            ProgressBar {
                has_children: true,
                ..bar(parent)
            },
            ProgressBar {
                parent_id: Some(parent),
                has_children: true,
                ..bar(child)
            },
            ProgressBar {
                parent_id: Some(child),
                ..bar(grandchild)
            },
            bar(unrelated),
        ] {
            storage.insert(bar.id, bar).await.unwrap();
        }

        drop(attached(parent, &storage, tx));

        let mut finished = Vec::new();
        for _ in 0..3 {
            let event = next_progress(&mut rx).await;
            assert_eq!(event.fraction, None);
            finished.push((event.progress_bar_id, event.status));
        }
        assert_eq!(
            finished,
            [
                (grandchild, ProgressStatus::Cancelled),
                (child, ProgressStatus::Cancelled),
                (parent, ProgressStatus::Done),
            ],
            "the bars left behind were cut short"
        );
        let left: Vec<_> = storage.list().await.into_iter().map(|bar| bar.id).collect();
        assert_eq!(left, [unrelated]);
    }

    /// The handles of a child and its parent dropped one after the other: whichever task of
    /// the two the runtime runs first, the child ends as it was dropped, and before the parent.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn a_child_dropped_before_its_parent_ends_before_it_as_done() {
        for round in 0..200 {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let storage = Arc::new(MapStorage::default());
            let sequencer = Arc::new(EventSequencer::new(Arc::new(ChannelEmitter(tx))));
            let [parent, child] = [(); 2].map(|()| Uuid::new_v4());
            for bar in [
                ProgressBar {
                    has_children: true,
                    ..bar(parent)
                },
                ProgressBar {
                    parent_id: Some(parent),
                    ..bar(child)
                },
            ] {
                storage.insert(bar.id, bar).await.unwrap();
            }
            let handle = |id| ProgressBarId {
                id,
                progress_storage: Some(storage.clone()),
                sequencer: Some(sequencer.clone()),
            };
            let (parent_handle, child_handle) = (handle(parent), handle(child));

            drop(child_handle);
            drop(parent_handle);

            let first = next_progress(&mut rx).await;
            let second = next_progress(&mut rx).await;
            assert_eq!(
                [
                    (first.progress_bar_id, first.status),
                    (second.progress_bar_id, second.status)
                ],
                [
                    (child, ProgressStatus::Done),
                    (parent, ProgressStatus::Done)
                ],
                "round {round}"
            );
            settle().await;
            assert!(rx.try_recv().is_err(), "round {round}: an event too many");
        }
    }

    #[tokio::test]
    async fn dropping_a_cancelled_progress_bar_id_emits_cancellation() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let storage = Arc::new(MapStorage::default());
        let id = Uuid::new_v4();
        storage.insert(id, bar(id)).await.unwrap();
        let bar_id = attached(id, &storage, tx);

        bar_id.cancel().await;
        assert!(rx.try_recv().is_err(), "cancel alone must not emit");
        drop(bar_id);

        let cancelled = next_progress(&mut rx).await;
        assert_eq!(cancelled.progress_bar_id, id);
        assert_eq!(cancelled.fraction, None);
        assert_eq!(cancelled.message, "Cancelled");
        assert_eq!(cancelled.status, ProgressStatus::Cancelled);
        assert!(storage.list().await.is_empty());
    }

    #[tokio::test]
    async fn mirrors_end_with_their_shared_bar_and_show_how_it_ended() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let storage = Arc::new(MapStorage::default());
        let [shared, waiting, mirror, gone_mirror] = [(); 4].map(|()| Uuid::new_v4());
        let shared_bar = ProgressBar {
            shared: true,
            mirrors: vec![gone_mirror, mirror],
            bytes_done: Some(7),
            bytes_total: Some(9),
            ..bar(shared)
        };
        for bar in [
            ProgressBar {
                has_children: true,
                ..bar(waiting)
            },
            ProgressBar::mirror(mirror, &bar(shared), waiting),
            shared_bar,
        ] {
            storage.insert(bar.id, bar).await.unwrap();
        }
        let shared_id = attached(shared, &storage, tx);

        shared_id.fail("No network").await;
        drop(shared_id);

        let failed = ProgressStatus::Failed {
            message: "No network".to_string(),
        };
        let mirrored = next_progress(&mut rx).await;
        assert_eq!(mirrored.progress_bar_id, mirror);
        assert_eq!(mirrored.mirror_of, Some(shared));
        assert_eq!(mirrored.parent_id, Some(waiting));
        assert_eq!(mirrored.status, failed);
        assert_eq!(
            (mirrored.bytes_done, mirrored.bytes_total),
            (Some(7), Some(9))
        );
        let own = next_progress(&mut rx).await;
        assert_eq!((own.progress_bar_id, own.status), (shared, failed));
        let left: Vec<_> = storage.list().await.into_iter().map(|bar| bar.id).collect();
        assert_eq!(left, [waiting], "the waiting bar goes on");
    }

    #[tokio::test]
    async fn failing_detached_progress_bar_id_emits_nothing() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let id = ProgressBarId {
            id: Uuid::new_v4(),
            progress_storage: None,
            sequencer: Some(Arc::new(EventSequencer::new(Arc::new(ChannelEmitter(tx))))),
        };

        id.fail("ignored").await;
        drop(id);
        settle().await;

        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn dropping_id_of_already_removed_bar_emits_nothing() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let bar_id = ProgressBarId {
            id: Uuid::new_v4(),
            progress_storage: Some(Arc::new(MapStorage::default())),
            sequencer: Some(Arc::new(EventSequencer::new(Arc::new(ChannelEmitter(tx))))),
        };

        drop(bar_id);
        settle().await;

        assert!(rx.try_recv().is_err());
    }
}
