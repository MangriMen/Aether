use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;

use crate::core::domain::error_chain;
use crate::features::{
    events::{EventEmitterExt, SharedEventEmitter, WarningEvent},
    instance::{InstanceId, InstanceStorage},
    process::app::ports::{SkippedHookNoticeService, TakeSkippedHookNoticesUseCasePort},
};

/// Tells the user that the post-exit hook of a game was not run.
///
/// The notices of the games found ended at start come before the window listens to events, so
/// they are held until the frontend takes them; every later one is sent as an event.
pub struct TakeSkippedHookNoticesUseCase {
    event_emitter: SharedEventEmitter,
    instance_storage: Arc<dyn InstanceStorage>,
    /// `Some` until the frontend first takes the notices.
    held: Mutex<Option<Vec<WarningEvent>>>,
}

impl TakeSkippedHookNoticesUseCase {
    pub fn new(
        event_emitter: SharedEventEmitter,
        instance_storage: Arc<dyn InstanceStorage>,
    ) -> Self {
        Self {
            event_emitter,
            instance_storage,
            held: Mutex::new(Some(Vec::new())),
        }
    }

    pub fn execute(&self) -> Vec<WarningEvent> {
        self.held
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
            .unwrap_or_default()
    }
}

#[async_trait]
impl SkippedHookNoticeService for TakeSkippedHookNoticesUseCase {
    async fn notify(&self, instance_id: &InstanceId) {
        // The instance may have been removed or broken: its id still tells which one it was.
        let instance_name = match self.instance_storage.get(instance_id).await {
            Ok(instance) => instance.name().to_string(),
            Err(err) => {
                tracing::debug!(
                    %instance_id,
                    error = %error_chain(&err),
                    "Could not read the name of the instance whose post-exit hook was skipped"
                );
                instance_id.to_string()
            }
        };
        let notice = WarningEvent::PostExitHookSkipped {
            instance_id: instance_id.clone(),
            instance_name,
        };
        {
            let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
            if let Some(held) = held.as_mut() {
                held.push(notice);
                return;
            }
        }
        self.event_emitter.emit_safe(notice).await;
    }
}

#[async_trait]
impl TakeSkippedHookNoticesUseCasePort for TakeSkippedHookNoticesUseCase {
    async fn execute(&self) -> Vec<WarningEvent> {
        self.execute()
    }
}
