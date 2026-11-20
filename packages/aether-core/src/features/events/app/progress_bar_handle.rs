use std::fmt;
use std::sync::Arc;

use log::error;
use uuid::Uuid;

use super::event_sequencer::EventSequencer;
use crate::ErrorKind;
use crate::features::events::{
    ProgressBar, ProgressBarStorage, ProgressEvent, ProgressFailure, ProgressStatus,
};

/// The handle of a bar: dropping it ends the bar, so there is one, and it is not `Clone`.
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

impl ProgressBarId {
    /// Marks the bar as failed: when the last use of it ends (the handle is dropped), listeners
    /// get a failed event with `error` instead of a completed one. The failure is logged here,
    /// once, whoever else reports it.
    ///
    /// Does nothing for an id that was never attached to a storage or whose bar is already gone.
    pub async fn fail(&self, error: impl Into<ErrorKind> + Send) {
        let failure = ProgressFailure::new(error);
        tracing::warn!(bar = %self.id, error = %failure.error().display_chain(), "Progress failed");
        self.end_as(ProgressStatus::Failed(failure)).await;
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
        events.push(bar.finished_event());
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
            events.push(mirror.finished_event());
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
                child.cut_short();
                ended.push(child.finished_event());
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

        // Outside a runtime nothing can end the bar now: it stays queued and ends with the
        // next bar that is dropped inside one.
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(Self::finish_dropped(sequencer));
        }
    }
}

#[cfg(test)]
mod tests {
    /// A failure for a test, by its text.
    fn failure(text: &str) -> ErrorKind {
        ErrorKind::CoreError(text.to_owned())
    }
    use std::time::Duration;

    use dashmap::DashMap;
    use dashmap::mapref::one::Ref as DashMapRef;
    use tokio::sync::mpsc;

    use crate::features::events::{
        Event, ProgressBarChange, ProgressBarStorage, ProgressBarStorageError,
        test_support::ChannelEmitter,
    };

    use super::*;
    use crate::features::events::{ProgressEventType, ProgressStage, domain::FAILED_MESSAGE};

    /// Minimal storage double: the handle must not depend on the real implementation.
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

    /// Lets the tasks the test spawned run. Only sound on a `current_thread` runtime and for
    /// work that is in memory, which a few turns of the scheduler finish; when a test must know
    /// that something is blocked, it waits for a signal from the blocked side instead.
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

        bar_id.fail(failure("Checksum mismatch")).await;
        assert!(rx.try_recv().is_err(), "fail alone must not emit");
        assert_eq!(
            storage.get(id).await.unwrap().status,
            ProgressStatus::Failed(ProgressFailure::new(failure("Checksum mismatch")))
        );
        drop(bar_id);

        let failed = next_progress(&mut rx).await;
        assert_eq!(failed.progress_bar_id, id);
        assert_eq!(failed.fraction, None);
        assert_eq!(failed.message, FAILED_MESSAGE);
        let ProgressStatus::Failed(failure) = failed.status else {
            panic!("expected a failed status, got {:?}", failed.status);
        };
        assert!(
            matches!(failure.error(), ErrorKind::CoreError(text) if text == "Checksum mismatch"),
            "the error itself is kept: {failure:?}"
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
            // The workers run apart from this task, so yielding would not wait for them: give an
            // event too many a moment to show.
            assert!(
                tokio::time::timeout(std::time::Duration::from_millis(2), rx.recv())
                    .await
                    .is_err(),
                "round {round}: an event too many"
            );
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

        shared_id.fail(failure("No network")).await;
        drop(shared_id);

        let failed = ProgressStatus::Failed(ProgressFailure::new(failure("No network")));
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

        id.fail(failure("ignored")).await;
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

    #[test]
    fn dropping_a_handle_outside_a_runtime_does_not_panic() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let storage = Arc::new(MapStorage::default());

        drop(attached(Uuid::new_v4(), &storage, tx));
    }
}
