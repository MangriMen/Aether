use std::sync::Arc;

use async_trait::async_trait;

use crate::features::instance::app::ports::ListInstancesUseCasePort;
use crate::features::instance::{Instance, InstanceError, InstanceStorage};

pub struct ListInstancesUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
}

impl ListInstancesUseCase {
    pub fn new(instance_storage: Arc<dyn InstanceStorage>) -> Self {
        Self { instance_storage }
    }
}

#[async_trait]
impl ListInstancesUseCasePort for ListInstancesUseCase {
    async fn execute(&self) -> Result<Vec<Instance>, InstanceError> {
        self.instance_storage.list().await
    }
}
