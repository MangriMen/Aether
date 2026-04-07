use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::events::{
    progress_service::ProgressService, EventEmitter, EventError, LauncherEvent, ProgressBar,
    ProgressBarId, ProgressBarStorage, ProgressBarStorageExt, ProgressEvent, ProgressEventType,
};

pub struct ProgressServiceImpl<E: EventEmitter, PS: ProgressBarStorage> {
    pub event_emitter: Arc<E>,
    progress_storage: Arc<PS>,
}

impl<E: EventEmitter, PS: ProgressBarStorage> ProgressServiceImpl<E, PS> {
    pub fn new(event_emitter: Arc<E>, progress_storage: Arc<PS>) -> Self {
        Self {
            event_emitter,
            progress_storage,
        }
    }

    async fn emit_progress_inner(
        &self,
        progress_bar_id: Uuid,
        fraction: Option<f64>,
        message: String,
        event_type: ProgressEventType,
    ) -> Result<(), EventError> {
        self.event_emitter
            .emit(
                LauncherEvent::Loading.as_str(),
                ProgressEvent {
                    fraction,
                    message,
                    event: event_type,
                    progress_bar_id,
                },
            )
            .await
    }
}

#[async_trait]
impl<E: EventEmitter, PS: ProgressBarStorage> ProgressService for ProgressServiceImpl<E, PS> {
    async fn init_progress(
        &self,
        event_type: ProgressEventType,
        total: f64,
        message: String,
    ) -> Result<ProgressBarId, EventError> {
        let progress_bar_id = ProgressBarId(Uuid::new_v4());

        self.progress_storage
            .insert(
                progress_bar_id.0,
                ProgressBar {
                    id: progress_bar_id.0,
                    message,
                    total,
                    current: 0.0,
                    last_sent: 0.0,
                    progress_type: event_type,
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
        let mut progress_bar = self.progress_storage.get(progress_bar_id.0).await?.clone();

        // Tick up loading bar
        progress_bar.current += increment_frac;

        let display_frac = progress_bar.current / progress_bar.total;
        let opt_display_frac = if display_frac >= 1.0 {
            None // by convention, when its done, we submit None
                 // any further updates will be ignored (also sending None)
        } else {
            Some(display_frac)
        };

        if f64::abs(display_frac - progress_bar.last_sent) > 0.005 {
            self.emit_progress_inner(
                progress_bar_id.0,
                opt_display_frac,
                message.unwrap_or(progress_bar.message.as_ref()).to_string(),
                progress_bar.progress_type.clone(),
            )
            .await?;
            progress_bar.last_sent = display_frac;
        }

        self.progress_storage
            .upsert(progress_bar_id.0, progress_bar)
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
}
