use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::events::app::ports::CancelProgressUseCasePort;
use crate::features::events::{EventError, ProgressService};

pub struct CancelProgressUseCase {
    progress_service: Arc<dyn ProgressService>,
}

impl CancelProgressUseCase {
    pub fn new(progress_service: Arc<dyn ProgressService>) -> Self {
        Self { progress_service }
    }
}

#[async_trait]
impl CancelProgressUseCasePort for CancelProgressUseCase {
    async fn execute(&self, progress_bar_id: Uuid) -> Result<(), EventError> {
        self.progress_service.cancel_progress(progress_bar_id).await
    }
}
