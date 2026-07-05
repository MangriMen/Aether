use std::sync::Arc;

use crate::features::instance::{Instance, InstanceError, InstanceStorage};

pub struct ListInstancesUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
}

impl ListInstancesUseCase {
    pub fn new(instance_storage: Arc<dyn InstanceStorage>) -> Self {
        Self { instance_storage }
    }

    pub async fn execute(&self) -> Result<Vec<Instance>, InstanceError> {
        self.instance_storage.list().await
    }
}
