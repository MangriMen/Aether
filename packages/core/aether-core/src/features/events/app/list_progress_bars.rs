use std::sync::Arc;

use async_trait::async_trait;

use crate::features::events::app::ports::ListProgressBarsUseCasePort;
use crate::features::events::{ProgressBar, ProgressBarStorage};

pub struct ListProgressBarsUseCase {
    progress_bar_storage: Arc<dyn ProgressBarStorage>,
}

impl ListProgressBarsUseCase {
    pub fn new(progress_bar_storage: Arc<dyn ProgressBarStorage>) -> Self {
        Self {
            progress_bar_storage,
        }
    }

    pub async fn execute(&self) -> Vec<ProgressBar> {
        ListProgressBarsUseCasePort::execute(self).await
    }
}

#[async_trait]
impl ListProgressBarsUseCasePort for ListProgressBarsUseCase {
    async fn execute(&self) -> Vec<ProgressBar> {
        self.progress_bar_storage.list().await
    }
}
