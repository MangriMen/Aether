use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use super::super::super::domain::ProcessError;
use super::super::ports::{KillProcessUseCasePort, ProcessStorage};

pub struct KillProcessUseCase {
    process_storage: Arc<dyn ProcessStorage>,
}

impl KillProcessUseCase {
    pub fn new(process_storage: Arc<dyn ProcessStorage>) -> Self {
        Self { process_storage }
    }

    pub async fn execute(&self, process_id: Uuid) -> Result<(), ProcessError> {
        KillProcessUseCasePort::execute(self, process_id).await
    }
}

#[async_trait]
impl KillProcessUseCasePort for KillProcessUseCase {
    async fn execute(&self, process_id: Uuid) -> Result<(), ProcessError> {
        self.process_storage.kill(process_id).await
    }
}
