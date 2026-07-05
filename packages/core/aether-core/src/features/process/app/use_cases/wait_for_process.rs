use std::sync::Arc;

use uuid::Uuid;

use super::super::super::domain::ProcessError;
use super::super::ports::ProcessStorage;

pub struct WaitForProcessUseCase {
    process_storage: Arc<dyn ProcessStorage>,
}

impl WaitForProcessUseCase {
    pub fn new(process_storage: Arc<dyn ProcessStorage>) -> Self {
        Self { process_storage }
    }

    pub async fn execute(&self, process_id: Uuid) -> Result<(), ProcessError> {
        self.process_storage.wait_for(process_id).await
    }
}
