use std::fmt;
use std::sync::Arc;

use log::error;
use uuid::Uuid;

use crate::features::events::{ProgressBarStorage, SharedEventEmitter};

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
    /// `Failed` once [`ProgressBarId::fail`] is called; the bar is gone when it is done.
    pub status: ProgressStatus,
    pub parent_id: Option<Uuid>,
    pub bytes_done: Option<u64>,
    pub bytes_total: Option<u64>,
    /// Transfer speed in bytes per second.
    pub speed: Option<f64>,
}

impl ProgressBar {
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
            bytes_done: self.bytes_done,
            bytes_total: self.bytes_total,
            speed: self.speed,
        }
    }
}

#[derive(Clone)]
pub struct ProgressBarId {
    pub id: Uuid,
    pub(crate) progress_storage: Option<Arc<dyn ProgressBarStorage>>,
    pub(crate) event_emitter: Option<SharedEventEmitter>,
}

impl fmt::Debug for ProgressBarId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProgressBarId")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct ProgressConfig<'a> {
    pub progress_bar_id: &'a ProgressBarId,
    pub total_progress: f64,
}

const COMPLETED_MESSAGE: &str = "Completed";

impl ProgressBarId {
    /// Marks the bar as failed: when the last use of it ends (the handle is dropped), listeners
    /// get a failed event with `message` instead of a completed one.
    ///
    /// Does nothing for an id that was never attached to a storage or whose bar is already gone.
    pub async fn fail(&self, message: impl Into<String> + Send) {
        let Some(storage) = &self.progress_storage else {
            return;
        };
        let mut bar = match storage.get(self.id).await {
            Ok(bar) => bar.clone(),
            Err(e) => {
                error!("Could not mark progress bar {} as failed: {e:?}", self.id);
                return;
            }
        };
        bar.status = ProgressStatus::Failed {
            message: message.into(),
        };
        if let Err(e) = storage.upsert(self.id, bar).await {
            error!("Could not mark progress bar {} as failed: {e:?}", self.id);
        }
    }

    /// Unregisters the bar and tells listeners that it is finished.
    ///
    /// Does nothing for an id that was never attached to a storage and an emitter.
    async fn finish(
        self_id: Uuid,
        storage: Arc<dyn ProgressBarStorage>,
        emitter: SharedEventEmitter,
    ) {
        let removed = match storage.remove(self_id).await {
            Ok(removed) => removed,
            Err(e) => {
                error!("Could not unregister progress bar {self_id}: {e:?}");
                return;
            }
        };
        let Some((_, bar)) = removed else {
            return;
        };

        let reached_percent = bar.current / bar.total * 100.0;
        let done = match &bar.status {
            ProgressStatus::Failed { message } => {
                bar.event(None, message.clone(), bar.status.clone())
            }
            ProgressStatus::Running | ProgressStatus::Done => {
                bar.event(None, COMPLETED_MESSAGE.to_string(), ProgressStatus::Done)
            }
        };

        if let Err(e) = emitter.emit(done.into()).await {
            error!(
                "Completion of progress bar {} (at {reached_percent:.2}%) was not delivered: {e:?}",
                bar.id
            );
        }
    }
}

/// Dropping the handle ends the bar's life: it is removed from storage and a final event
/// with no fraction is sent, done or, after [`ProgressBarId::fail`], failed.
impl Drop for ProgressBarId {
    fn drop(&mut self) {
        let (Some(storage), Some(emitter)) =
            (self.progress_storage.take(), self.event_emitter.take())
        else {
            return;
        };
        let id = self.id;

        tokio::spawn(Self::finish(id, storage, emitter));
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use dashmap::DashMap;
    use dashmap::mapref::one::Ref as DashMapRef;
    use tokio::sync::mpsc;

    use crate::features::events::{
        Event, EventEmitter, EventError, ProgressBarStorage, ProgressBarStorageError,
    };

    use super::*;

    struct ChannelEmitter(mpsc::UnboundedSender<Event>);

    #[async_trait::async_trait]
    impl EventEmitter<Event> for ChannelEmitter {
        async fn emit(&self, event: Event) -> Result<(), EventError> {
            let _ = self.0.send(event);
            Ok(())
        }

        fn listen(&self, _handler: Box<dyn Fn(Event) + Send + Sync + 'static>) {}
    }

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

        async fn upsert(
            &self,
            progress_bar_id: Uuid,
            progress_bar: ProgressBar,
        ) -> Result<(), ProgressBarStorageError> {
            self.0.insert(progress_bar_id, progress_bar);
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
            id,
            message: "Downloading".to_string(),
            total: 10.0,
            current: 4.0,
            last_sent: 0.0,
            progress_type: jar_event(),
            stage: None,
            status: ProgressStatus::Running,
            parent_id: None,
            bytes_done: None,
            bytes_total: None,
            speed: None,
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
            event_emitter: Some(Arc::new(ChannelEmitter(tx))),
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
            event_emitter: Some(Arc::new(ChannelEmitter(tx))),
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
    async fn failing_detached_progress_bar_id_does_nothing() {
        let id = ProgressBarId {
            id: Uuid::new_v4(),
            progress_storage: None,
            event_emitter: None,
        };
        id.fail("ignored").await;
    }

    #[tokio::test]
    async fn dropping_id_of_already_removed_bar_emits_nothing() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let bar_id = ProgressBarId {
            id: Uuid::new_v4(),
            progress_storage: Some(Arc::new(MapStorage::default())),
            event_emitter: Some(Arc::new(ChannelEmitter(tx))),
        };

        drop(bar_id);
        settle().await;

        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn dropping_detached_progress_bar_id_emits_nothing() {
        let id = ProgressBarId {
            id: Uuid::new_v4(),
            progress_storage: None,
            event_emitter: None,
        };
        drop(id);
        settle().await;
    }
}
