use std::sync::Arc;

use crate::features::update::{UpdateService, UpdateStatus};

pub struct CheckForUpdatesUseCase<US: UpdateService> {
    update_service: Arc<US>,
}

impl<US: UpdateService> CheckForUpdatesUseCase<US> {
    pub fn new(update_service: Arc<US>) -> Self {
        Self { update_service }
    }

    pub async fn execute(&self) -> Result<Option<UpdateStatus>, String> {
        self.update_service.check().await
    }
}
