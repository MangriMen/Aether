use crate::features::{
    instance::{Instance, InstanceError},
    settings::{EditHooks, MemorySettings, WindowSettings},
};

#[derive(Debug)]
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

impl EditInstance {
    /// Applies changes to the domain instance.
    /// Consumes `self` to avoid memory allocations during field migration.
    pub fn apply_to(self, instance: &mut Instance) -> Result<bool, InstanceError> {
        let mut is_changed = false;

        // Name
        if let Some(name) = self.name {
            is_changed |= instance.rename(name)?;
        }

        // Java Path
        if let Some(active) = self.override_java_path {
            is_changed |= instance.update_java_path_active(active);
        }
        if let Some(data) = self.java_path {
            is_changed |= instance.update_java_path_data(data);
        }

        // Launch Args
        if let Some(active) = self.override_launch_args {
            is_changed |= instance.update_launch_args_active(active);
        }
        if let Some(data) = self.launch_args {
            is_changed |= instance.update_launch_args_data(data);
        }

        // Env Vars
        if let Some(active) = self.override_env_vars {
            is_changed |= instance.update_env_vars_active(active);
        }
        if let Some(data) = self.env_vars {
            is_changed |= instance.update_env_vars_data(data);
        }

        // Memory
        if let Some(active) = self.override_memory {
            is_changed |= instance.update_memory_active(active);
        }
        if let Some(data) = self.memory {
            is_changed |= instance.update_memory_data(data);
        }

        // Window Settings
        if let Some(active) = self.override_window_settings {
            is_changed |= instance.update_window_active(active);
        }
        if let Some(data) = self.window {
            is_changed |= instance.update_window_data(data);
        }

        // Hooks
        if let Some(active) = self.override_hooks {
            is_changed |= instance.update_hooks_active(active);
        }
        if let Some(edit_hooks) = self.hooks {
            is_changed |= instance.update_hooks_data(
                edit_hooks.pre_launch,
                edit_hooks.wrapper,
                edit_hooks.post_exit,
            );
        }

        Ok(is_changed)
    }
}
