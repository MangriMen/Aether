use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::features::instance::app::ports::EditInstanceUseCasePort;
use crate::features::instance::{
    EditInstance, Instance, InstanceError, InstanceId, InstanceStorage,
    domain::{InstanceField, InstanceValidationErrorReason},
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
        validate_edit(&edit_instance)?;

        let mut edit_instance = Some(edit_instance);
        self.instance_storage
            .update(&instance_id, &mut |instance| {
                let Some(edit) = edit_instance.take() else {
                    return Ok(false);
                };
                let changed = edit.apply_to(instance)?;
                if changed {
                    instance.modified = Utc::now();
                }
                Ok(changed)
            })
            .await
    }
}

fn validate_edit(edit: &EditInstance) -> Result<(), InstanceError> {
    if let Some(name) = &edit.name {
        validate_name(name)?;
    }

    Ok(())
}

fn validate_name(name: &str) -> Result<(), InstanceError> {
    if name.trim().is_empty() {
        return Err(InstanceError::ValidationError {
            field: InstanceField::Name,
            reason: InstanceValidationErrorReason::CannotBeEmpty,
        });
    }
    Ok(())
}
