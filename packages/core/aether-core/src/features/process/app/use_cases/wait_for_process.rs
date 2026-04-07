use std::sync::Arc;

use uuid::Uuid;

use crate::features::process::{ProcessError, ProcessStorage};

pub struct WaitForProcessUseCase<PS: ProcessStorage> {
    process_storage: Arc<PS>,
}

impl<PS: ProcessStorage> WaitForProcessUseCase<PS> {
    pub fn new(process_storage: Arc<PS>) -> Self {
        Self { process_storage }
    }

    pub async fn execute(&self, instance_id: Uuid) -> Result<(), ProcessError> {
        self.process_storage.wait_for(instance_id).await
    }
}
