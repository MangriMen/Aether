use crate::features::settings::{
    DefaultInstanceSettings, MemorySettings, Settings, WindowSettings,
};

#[derive(Debug)]
pub struct EditSettings {
    pub max_concurrent_downloads: usize,
}

impl EditSettings {
    pub fn apply_to(self, settings: &mut Settings) -> bool {
        if settings.max_concurrent_downloads() != self.max_concurrent_downloads {
            settings.set_max_concurrent_downloads(self.max_concurrent_downloads);
            return true;
        }
        false
    }
}

#[derive(Debug)]
pub struct EditDefaultInstanceSettings {
    pub launch_args: Option<Vec<String>>,
    pub env_vars: Option<Vec<(String, String)>>,
    pub memory: Option<MemorySettings>,
    pub window: Option<WindowSettings>,
    pub hooks: Option<EditHooks>,
}

#[derive(Debug)]
pub struct EditHooks {
    pub pre_launch: Option<String>,

    pub wrapper: Option<String>,

    pub post_exit: Option<String>,
}

impl EditDefaultInstanceSettings {
    pub fn apply_to(self, settings: &mut DefaultInstanceSettings) -> bool {
        let mut is_changed = false;

        if let Some(launch_args) = self.launch_args {
            is_changed |= settings.update_launch_args(launch_args);
        }

        if let Some(env_vars) = self.env_vars {
            is_changed |= settings.update_env_vars(env_vars);
        }

        if let Some(memory) = self.memory {
            is_changed |= settings.update_memory(memory);
        }

        if let Some(window) = self.window {
            is_changed |= settings.update_window(window);
        }

        if let Some(edit_hooks) = self.hooks {
            is_changed |= edit_hooks.apply_to(settings);
        }

        is_changed
    }
}

impl EditHooks {
    pub fn apply_to(self, settings: &mut DefaultInstanceSettings) -> bool {
        settings.update_hooks(self.pre_launch, self.wrapper, self.post_exit)
    }
}
