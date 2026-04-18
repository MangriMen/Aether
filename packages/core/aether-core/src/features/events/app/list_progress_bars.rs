use std::sync::Arc;

use crate::features::events::{ProgressBar, ProgressBarStorage};

pub struct ListProgressBarsUseCase<PBS: ProgressBarStorage> {
    progress_bar_storage: Arc<PBS>,
}

impl<PBS: ProgressBarStorage> ListProgressBarsUseCase<PBS> {
    pub fn new(progress_bar_storage: Arc<PBS>) -> Self {
        Self {
            progress_bar_storage,
        }
    }

    pub async fn execute(&self) -> Vec<ProgressBar> {
        self.progress_bar_storage.list().await
    }
}
