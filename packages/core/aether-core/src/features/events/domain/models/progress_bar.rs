use log::error;
use uuid::Uuid;

use crate::{core::app::AetherContainer, features::events::ProgressBarStorage};

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

#[derive(Debug, Clone)]
pub struct ProgressBarId(pub Uuid);

#[derive(Debug)]
pub struct ProgressConfig<'a> {
    pub progress_bar_id: &'a ProgressBarId,
    pub total_progress: f64,
}

// When Loading bar id is dropped, we should remove it from the state
impl Drop for ProgressBarId {
    fn drop(&mut self) {
        let progress_bar_id = self.0;
        tokio::spawn(async move {
            let container = AetherContainer::try_get();

            match container {
                Some(container) => {
                    let progress_bar_storage = container.progress_bar_storage();
                    let event_emitter = container.event_emitter();

                    let removed_progress_bar =
                        progress_bar_storage.remove(progress_bar_id).await.unwrap();

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
                }
                None => error!("Failed to get AetherContainer in ProgressBarId::drop"),
            }
        });
    }
}
