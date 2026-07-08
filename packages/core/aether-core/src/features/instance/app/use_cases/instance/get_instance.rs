use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::app::ports::GetInstanceUseCasePort;
use crate::features::instance::{Instance, InstanceError, InstanceStorage};

pub struct GetInstanceUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
}

impl GetInstanceUseCase {
    pub fn new(instance_storage: Arc<dyn InstanceStorage>) -> Self {
        Self { instance_storage }
    }

    pub async fn execute(&self, instance_id: String) -> Result<Instance, InstanceError> {
        GetInstanceUseCasePort::execute(self, instance_id).await
    }
}

#[async_trait]
impl GetInstanceUseCasePort for GetInstanceUseCase {
    async fn execute(&self, instance_id: String) -> Result<Instance, InstanceError> {
        self.instance_storage.get(&instance_id).await
    }
}
