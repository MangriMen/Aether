use std::sync::Arc;

use uuid::Uuid;

use super::super::super::domain::ProcessError;
use super::super::ports::ProcessStorage;

pub struct WaitForProcessUseCase<PS: ProcessStorage> {
    process_storage: Arc<PS>,
}

impl<PS: ProcessStorage> WaitForProcessUseCase<PS> {
    pub fn new(process_storage: Arc<PS>) -> Self {
        Self { process_storage }
    }

    pub async fn execute(&self, process_id: Uuid) -> Result<(), ProcessError> {
        self.process_storage.wait_for(process_id).await
    }
}
