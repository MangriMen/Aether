use serde::{Deserialize, Serialize};

use crate::features::settings::{Hooks, MemorySettings, WindowSize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchSettings {
    pub launch_args: Vec<String>,
    pub env_vars: Vec<(String, String)>,
    pub memory: MemorySettings,
    pub game_resolution: WindowSize,
    pub hooks: Hooks,
}
