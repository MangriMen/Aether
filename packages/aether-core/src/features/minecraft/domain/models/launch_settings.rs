use crate::features::settings::{Hooks, MemorySettings, WindowSettings};

#[derive(Debug, Clone)]
pub struct LaunchSettings {
    pub launch_args: Vec<String>,

    pub env_vars: Vec<(String, String)>,

    pub memory: MemorySettings,

    pub window: WindowSettings,

    pub hooks: Hooks,
}
