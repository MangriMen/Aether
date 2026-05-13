use serde::{Deserialize, Serialize};

use crate::features::settings::{Hooks, MemorySettings, WindowSettings};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchSettings {
    pub launch_args: Vec<String>,

    pub env_vars: Vec<(String, String)>,

    pub memory: MemorySettings,

    pub window: WindowSettings,

    pub hooks: Hooks,
}
