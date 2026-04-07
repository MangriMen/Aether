use serde::{Deserialize, Serialize};

use crate::features::settings::{DefaultInstanceSettings, Hooks, MemorySettings, WindowSize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditDefaultInstanceSettings {
    pub launch_args: Option<Vec<String>>,
    pub env_vars: Option<Vec<(String, String)>>,
    pub memory: Option<MemorySettings>,
    pub game_resolution: Option<WindowSize>,
    pub hooks: Option<EditHooks>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditHooks {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub pre_launch: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub wrapper: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub post_exit: Option<Option<String>>,
}

impl EditDefaultInstanceSettings {
    pub fn apply_to(self, settings: &mut DefaultInstanceSettings) -> bool {
        let mut is_changed = false;

        if let Some(launch_args) = self.launch_args {
            settings.set_launch_args(launch_args);
            is_changed = true;
        }

        if let Some(env_vars) = self.env_vars {
            settings.set_env_vars(env_vars);
            is_changed = true;
        }

        if let Some(memory) = self.memory {
            settings.set_memory(memory);
            is_changed = true;
        }

        if let Some(game_resolution) = self.game_resolution {
            settings.set_resolution(game_resolution);
            is_changed = true;
        }

        if let Some(edit_hooks) = self.hooks {
            edit_hooks.apply_to(settings.hooks_mut());
            is_changed = true;
        }

        is_changed
    }
}

impl EditHooks {
    pub fn apply_to(&self, hooks: &mut Hooks) {
        if let Some(val) = &self.pre_launch {
            hooks.update_pre_launch(val.clone());
        }
        if let Some(val) = &self.wrapper {
            hooks.update_wrapper(val.clone());
        }
        if let Some(val) = &self.post_exit {
            hooks.update_post_exit(val.clone());
        }
    }
}
