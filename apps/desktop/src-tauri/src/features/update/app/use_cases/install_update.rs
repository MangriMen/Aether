use std::sync::Arc;

use crate::features::update::UpdateService;

pub struct InstallUpdateUseCase<US: UpdateService> {
    update_service: Arc<US>,
}

impl<US: UpdateService> InstallUpdateUseCase<US> {
    pub fn new(update_service: Arc<US>) -> Self {
        Self { update_service }
    }

    pub async fn execute(&self) -> Result<(), String> {
        self.update_service.install().await
    }
}
