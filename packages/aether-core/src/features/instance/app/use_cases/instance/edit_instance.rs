use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::features::instance::app::ports::EditInstanceUseCasePort;
use crate::features::instance::{
    EditInstance, Instance, InstanceError, InstanceId, InstanceStorage,
};

pub struct EditInstanceUseCase {
    instance_storage: Arc<dyn InstanceStorage>,
}

impl EditInstanceUseCase {
    pub fn new(instance_storage: Arc<dyn InstanceStorage>) -> Self {
        Self { instance_storage }
    }
}

#[async_trait]
impl EditInstanceUseCasePort for EditInstanceUseCase {
    async fn execute(
        &self,
        instance_id: InstanceId,
        edit_instance: EditInstance,
    ) -> Result<Instance, InstanceError> {
        // The name is checked by `Instance::rename`, the only place that knows what a name is.
        self.instance_storage
            .update(
                &instance_id,
                Box::new(move |instance| {
                    let changed = edit_instance.apply_to(instance)?;
                    if changed {
                        instance.modified = Utc::now();
                    }
                    Ok(changed)
                }),
            )
            .await
    }
}
