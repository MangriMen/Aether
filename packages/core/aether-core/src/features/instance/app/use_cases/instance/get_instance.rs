use std::sync::Arc;

use crate::features::instance::{Instance, InstanceError, InstanceStorage};

pub struct GetInstanceUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
}

impl GetInstanceUseCase {
    pub fn new(instance_storage: Arc<dyn InstanceStorage>) -> Self {
        Self { instance_storage }
    }

    pub async fn execute(&self, instance_id: String) -> Result<Instance, InstanceError> {
        self.instance_storage.get(&instance_id).await
    }
}
