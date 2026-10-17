use std::fmt;
use std::sync::Arc;

use log::error;
use uuid::Uuid;

use crate::features::events::{ProgressBarStorage, SharedEventEmitter};

use super::{ProgressEvent, ProgressEventType};

#[derive(Debug, Clone)]
pub struct ProgressBar {
    // Identifies the bar; the live values are in the storage, not in copies of this struct.
    pub id: Uuid,
    pub message: String,
    pub total: f64,
    pub current: f64,
    pub last_sent: f64,
    pub progress_type: ProgressEventType,
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
        let done = ProgressEvent {
            fraction: None,
            message: COMPLETED_MESSAGE.to_string(),
            event: bar.progress_type,
            progress_bar_id: bar.id,
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
/// with no fraction is sent.
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

    use tokio::sync::mpsc;

    use crate::features::events::infra::InMemoryProgressBarStorage;
    use crate::features::events::{
        Event, EventEmitter, EventError, ProgressBarStorage, ProgressService, ProgressServiceImpl,
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

    fn jar_event() -> ProgressEventType {
        ProgressEventType::JavaDownload { version: 21 }
    }

    #[tokio::test]
    async fn dropping_progress_bar_id_removes_bar_and_emits_completion() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let storage = Arc::new(InMemoryProgressBarStorage::default());
        let service = ProgressServiceImpl::new(Arc::new(ChannelEmitter(tx)), storage.clone());

        let bar_id = service
            .init_progress(jar_event(), 10.0, "Downloading".to_string())
            .await
            .unwrap();
        let id = bar_id.id;
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
        assert_eq!(done.event, jar_event());
        assert!(storage.list().await.is_empty());
    }

    #[tokio::test]
    async fn dropping_detached_progress_bar_id_does_nothing() {
        let id = ProgressBarId {
            id: Uuid::new_v4(),
            progress_storage: None,
            event_emitter: None,
        };
        drop(id);
        tokio::task::yield_now().await;
    }
}
