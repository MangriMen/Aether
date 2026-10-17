use std::fmt;
use std::sync::Arc;

use log::error;
use uuid::Uuid;

use crate::features::events::{ProgressBarStorage, SharedEventEmitter};

use super::{ProgressEvent, ProgressEventType};

#[derive(Debug, Clone)]
pub struct ProgressBar {
    // id not be used directly by external functions as it may not reflect the current state
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

// When Loading bar id is dropped, we should remove it from the state
impl Drop for ProgressBarId {
    fn drop(&mut self) {
        let progress_bar_id = self.id;
        let progress_storage = self.progress_storage.take();
        let event_emitter = self.event_emitter.take();

        tokio::spawn(async move {
            let Some(progress_bar_storage) = progress_storage else {
                return;
            };
            let Some(event_emitter) = event_emitter else {
                return;
            };

            let removed_progress_bar = match progress_bar_storage.remove(progress_bar_id).await {
                Ok(removed) => removed,
                Err(e) => {
                    error!("Failed to remove progress bar {progress_bar_id}: {e:?}");
                    return;
                }
            };

            if let Some((_, progress_bar)) = removed_progress_bar {
                let completion_event = ProgressEvent {
                    fraction: None,
                    message: "Completed".to_string(),
                    event: progress_bar.progress_type,
                    progress_bar_id: progress_bar.id,
                };

                if let Err(e) = event_emitter.emit(completion_event.into()).await {
                    error!(
                        "Exited at {:.2}% for progress bar: {}: {:?}",
                        (progress_bar.current / progress_bar.total) * 100.0,
                        progress_bar.id,
                        e
                    );
                }
            }
        });
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
