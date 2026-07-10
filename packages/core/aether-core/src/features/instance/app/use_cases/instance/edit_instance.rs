use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::features::instance::app::ports::EditInstanceUseCasePort;
use crate::features::instance::{
    EditInstance, Instance, InstanceError, InstanceStorage,
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
        instance_id: String,
        edit_instance: EditInstance,
    ) -> Result<Instance, InstanceError> {
        validate_edit(&edit_instance)?;

        let mut instance = self.instance_storage.get(&instance_id).await?;

        if edit_instance.apply_to(&mut instance)? {
            instance.modified = Utc::now();
            self.instance_storage.upsert(&instance).await?;
        }

        Ok(instance)
    }
}

fn validate_edit(edit: &EditInstance) -> Result<(), InstanceError> {
    if let Some(name) = &edit.name {
        validate_name(name)?;
    }

    Ok(())
}

fn validate_name(name: &str) -> Result<(), InstanceError> {
    if name.is_empty() {
        return Err(InstanceError::ValidationError {
            field: InstanceField::Name,
            reason: InstanceValidationErrorReason::CannotBeEmpty,
        });
    }
    Ok(())
}
