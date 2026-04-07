use std::sync::Arc;

use uuid::Uuid;

use crate::features::process::{ProcessError, ProcessStorage};

pub struct KillProcessUseCase<PS: ProcessStorage> {
    process_storage: Arc<PS>,
}

impl<PS: ProcessStorage> KillProcessUseCase<PS> {
    pub fn new(process_storage: Arc<PS>) -> Self {
        Self { process_storage }
    }
    pub async fn execute(&self, instance_id: Uuid) -> Result<(), ProcessError> {
        self.process_storage.kill(instance_id).await
    }
}
