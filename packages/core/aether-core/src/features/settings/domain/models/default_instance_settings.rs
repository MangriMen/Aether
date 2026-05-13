use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DefaultInstanceSettings {
    launch_args: Vec<String>,
    env_vars: Vec<(String, String)>,

    memory: MemorySettings,

    window: WindowSettings,

    hooks: Hooks,
}

impl DefaultInstanceSettings {
    pub fn new(
        launch_args: Vec<String>,
        env_vars: Vec<(String, String)>,
        memory: MemorySettings,
        window: WindowSettings,
        hooks: Hooks,
    ) -> Self {
        Self {
            launch_args,
            env_vars,
            memory,
            window,
            hooks,
        }
    }

    pub fn launch_args(&self) -> &[String] {
        &self.launch_args
    }

    pub fn env_vars(&self) -> &[(String, String)] {
        &self.env_vars
    }

    pub fn memory(&self) -> &MemorySettings {
        &self.memory
    }

    pub fn window(&self) -> &WindowSettings {
        &self.window
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

    pub fn set_window(&mut self, window: WindowSettings) {
        self.window = window;
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

impl MemorySettings {
    pub fn new(maximum: u32) -> Self {
        Self { maximum }
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

impl WindowSize {
    pub fn new(width: u16, height: u16) -> Self {
        Self(width, height)
    }

    pub fn width(&self) -> u16 {
        self.0
    }

    pub fn height(&self) -> u16 {
        self.1
    }
}

/// Represents the visual configuration of the game window.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WindowSettings {
    /// Whether the game should start in full-screen mode.
    force_fullscreen: bool,
    /// The specific width and height settings for the game window.
    game_resolution: WindowSize,
}

impl WindowSettings {
    pub fn new(force_fullscreen: bool, game_resolution: WindowSize) -> Self {
        Self {
            force_fullscreen,
            game_resolution,
        }
    }

    pub fn force_fullscreen(&self) -> bool {
        self.force_fullscreen
    }

    pub fn game_resolution(&self) -> WindowSize {
        self.game_resolution
    }

    pub fn set_fullscreen(&mut self, enabled: bool) {
        self.force_fullscreen = enabled;
    }

    pub fn set_resolution(&mut self, resolution: WindowSize) {
        self.game_resolution = resolution;
    }
}

/// A struct representing various hooks configured for specific actions
/// in the application lifecycle.
///
/// Each field contains a command or script path. An empty string indicates
/// that no hook is defined for that specific lifecycle stage.
///
/// # Fields
/// - `pre_launch`: A command or script to be executed before the game starts.
/// - `wrapper`: A command or script used to wrap the game's execution process.
/// - `post_exit`: A command or script to be executed after the game process terminates.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Hooks {
    pre_launch: String,
    wrapper: String,
    post_exit: String,
}

impl Hooks {
    pub fn new(pre_launch: String, wrapper: String, post_exit: String) -> Self {
        Self {
            pre_launch,
            wrapper,
            post_exit,
        }
    }

    pub fn pre_launch(&self) -> &str {
        &self.pre_launch
    }
    pub fn wrapper(&self) -> &str {
        &self.wrapper
    }
    pub fn post_exit(&self) -> &str {
        &self.post_exit
    }

    pub fn set_pre_launch(&mut self, val: String) {
        self.pre_launch = val;
    }
    pub fn set_wrapper(&mut self, val: String) {
        self.wrapper = val;
    }
    pub fn set_post_exit(&mut self, val: String) {
        self.post_exit = val;
    }
}
