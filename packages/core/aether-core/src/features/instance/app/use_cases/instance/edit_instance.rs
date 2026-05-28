use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::features::{
    instance::{Instance, InstanceError, InstanceStorage},
    settings::{EditHooks, MemorySettings, WindowSettings},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditInstance {
    pub name: Option<String>,

    pub override_java_path: Option<bool>,
    pub java_path: Option<String>,

    pub override_launch_args: Option<bool>,
    pub launch_args: Option<Vec<String>>,

    pub override_env_vars: Option<bool>,
    pub env_vars: Option<Vec<(String, String)>>,

    pub override_memory: Option<bool>,
    pub memory: Option<MemorySettings>,

    pub override_window_settings: Option<bool>,
    pub window: Option<WindowSettings>,

    pub override_hooks: Option<bool>,
    pub hooks: Option<EditHooks>,
}

pub struct EditInstanceUseCase<IS> {
    instance_storage: Arc<IS>,
}

impl<IS: InstanceStorage> EditInstanceUseCase<IS> {
    pub fn new(instance_storage: Arc<IS>) -> Self {
        Self { instance_storage }
    }

    pub async fn execute(
        &self,
        instance_id: String,
        edit_instance: EditInstance,
    ) -> Result<Instance, InstanceError> {
        validate_edit(&edit_instance)?;

        let mut instance = self.instance_storage.get(&instance_id).await?;

        apply_edit_changes(&mut instance, &edit_instance)?;

        self.instance_storage.upsert(&instance).await?;

        Ok(instance)
    }
}

fn apply_edit_changes(
    instance: &mut Instance,
    edit_instance: &EditInstance,
) -> Result<(), InstanceError> {
    if let Some(name) = &edit_instance.name {
        instance.rename(name.clone())?;
    }

    // Java Path
    if let Some(active) = edit_instance.override_java_path {
        instance.java_path.is_active = active;
    }
    if let Some(data) = &edit_instance.java_path {
        instance.java_path.data.clone_from(data);
    }

    // Launch Args
    if let Some(active) = edit_instance.override_launch_args {
        instance.launch_args.is_active = active;
    }
    if let Some(data) = &edit_instance.launch_args {
        instance.launch_args.data.clone_from(data);
    }

    // Env Vars
    if let Some(active) = edit_instance.override_env_vars {
        instance.env_vars.is_active = active;
    }
    if let Some(data) = &edit_instance.env_vars {
        instance.env_vars.data.clone_from(data);
    }

    // Memory
    if let Some(active) = edit_instance.override_memory {
        instance.memory.is_active = active;
    }
    if let Some(data) = edit_instance.memory {
        instance.memory.data = data;
    }

    // Window Settings
    if let Some(active) = edit_instance.override_window_settings {
        instance.window.is_active = active;
    }
    if let Some(data) = &edit_instance.window {
        instance.window.data.clone_from(data);
    }

    // Hooks
    if let Some(active) = edit_instance.override_hooks {
        instance.hooks.is_active = active;
    }
    if let Some(edit_hooks) = &edit_instance.hooks {
        edit_hooks.apply_to(&mut instance.hooks.data);
    }

    instance.modified = Utc::now();

    Ok(())
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
            field: "name".to_owned(),
            reason: "name cannot be empty".to_owned(),
        });
    }
    Ok(())
}
