use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DefaultInstanceSettings {
    launch_args: Vec<String>,
    env_vars: Vec<(String, String)>,

    memory: MemorySettings,
    game_resolution: WindowSize,

    hooks: Hooks,
}

impl DefaultInstanceSettings {
    pub fn launch_args(&self) -> &[String] {
        &self.launch_args
    }

    pub fn env_vars(&self) -> &[(String, String)] {
        &self.env_vars
    }

    pub fn memory(&self) -> MemorySettings {
        self.memory
    }

    pub fn game_resolution(&self) -> WindowSize {
        self.game_resolution
    }

    pub fn hooks(&self) -> &Hooks {
        &self.hooks
    }

    pub fn hooks_mut(&mut self) -> &mut Hooks {
        &mut self.hooks
    }

    pub fn set_launch_args(&mut self, launch_args: Vec<String>) {
        self.launch_args = launch_args;
    }

    pub fn set_env_vars(&mut self, env_vars: Vec<(String, String)>) {
        self.env_vars = env_vars;
    }

    pub fn set_memory(&mut self, memory: MemorySettings) {
        self.memory = memory;
    }

    pub fn set_resolution(&mut self, resolution: WindowSize) {
        self.game_resolution = resolution;
    }
}

/// Memory usage settings for Java.
///
/// Used to define the maximum amount of memory that can be allocated
/// to the JVM when launching a game.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct MemorySettings {
    /// Maximum amount of RAM in megabytes.
    ///
    /// Typically corresponds to the `-Xmx` parameter when starting the JVM.
    pub maximum: u32,
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self { maximum: 2048 }
    }
}

/// A 2D size, represented by a tuple of two integers
///
/// First is the width, second is the height of the window (width, height)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct WindowSize(pub u16, pub u16);

impl Default for WindowSize {
    fn default() -> Self {
        Self(960, 540)
    }
}

/// A struct representing various hooks that can be configured for specific actions
/// in the application lifecycle.
///
/// # Fields
/// - `pre_launch`: An optional string representing a command or script to be executed
///   before the game launches.
/// - `wrapper`: An optional string representing a wrapper command or script to be used
///   during the game's execution.
/// - `post_exit`: An optional string representing a command or script to be executed
///   after the game exits.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Hooks {
    pre_launch: Option<String>,
    wrapper: Option<String>,
    post_exit: Option<String>,
}

impl Hooks {
    pub fn new(
        pre_launch: Option<String>,
        wrapper: Option<String>,
        post_exit: Option<String>,
    ) -> Self {
        Self {
            pre_launch,
            wrapper,
            post_exit,
        }
    }

    pub fn pre_launch(&self) -> Option<&String> {
        self.pre_launch.as_ref()
    }
    pub fn wrapper(&self) -> Option<&String> {
        self.wrapper.as_ref()
    }
    pub fn post_exit(&self) -> Option<&String> {
        self.post_exit.as_ref()
    }

    pub fn update_pre_launch(&mut self, val: Option<String>) {
        self.pre_launch = val;
    }
    pub fn update_wrapper(&mut self, val: Option<String>) {
        self.wrapper = val;
    }
    pub fn update_post_exit(&mut self, val: Option<String>) {
        self.post_exit = val;
    }
}
