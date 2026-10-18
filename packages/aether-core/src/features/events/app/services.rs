use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::events::{
    EventError, ProgressBar, ProgressBarId, ProgressBarStorage, ProgressBarStorageExt,
    ProgressEventType, ProgressStage, ProgressStatus, SharedEventEmitter,
    app::ports::ProgressService,
};

pub struct ProgressServiceImpl<PS: ProgressBarStorage + 'static> {
    pub event_emitter: SharedEventEmitter,
    progress_storage: Arc<PS>,
}

impl<PS: ProgressBarStorage + 'static> ProgressServiceImpl<PS> {
    pub fn new(event_emitter: SharedEventEmitter, progress_storage: Arc<PS>) -> Self {
        Self {
            event_emitter,
            progress_storage,
        }
    }

    async fn emit_progress_inner(
        &self,
        progress_bar: &ProgressBar,
        fraction: Option<f64>,
        message: String,
    ) -> Result<(), EventError> {
        self.event_emitter
            .emit(
                progress_bar
                    .event(fraction, message, ProgressStatus::Running)
                    .into(),
            )
            .await
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
            self.emit_progress_inner(
                &progress_bar,
                shown_fraction(display_frac),
                message.unwrap_or(progress_bar.message.as_ref()).to_string(),
            )
            .await?;
            progress_bar.last_sent = display_frac;
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
        let display_frac = progress_bar.current / progress_bar.total;
        self.emit_progress_inner(
            &progress_bar,
            shown_fraction(display_frac),
            progress_bar.message.clone(),
        )
        .await?;
        progress_bar.last_sent = display_frac;

        self.progress_storage
            .upsert(progress_bar_id.id, progress_bar)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::mpsc;

    use crate::features::events::{
        Event, EventEmitter, ProgressEvent, infra::InMemoryProgressBarStorage,
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
}
