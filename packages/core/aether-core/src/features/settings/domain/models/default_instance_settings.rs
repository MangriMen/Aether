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
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── DefaultInstanceSettings ──

    #[test]
    fn should_create_default_instance_settings_with_defaults() {
        let settings = DefaultInstanceSettings::default();

        assert!(settings.launch_args().is_empty());
        assert!(settings.env_vars().is_empty());
        assert_eq!(settings.memory().maximum, 2048);
        assert!(!settings.window().force_fullscreen());
        assert_eq!(settings.window().game_resolution().width(), 960);
        assert_eq!(settings.window().game_resolution().height(), 540);
        assert!(settings.hooks().pre_launch().is_empty());
        assert!(settings.hooks().wrapper().is_empty());
        assert!(settings.hooks().post_exit().is_empty());
    }

    #[test]
    fn should_construct_default_instance_settings_with_new() {
        let settings = DefaultInstanceSettings::new(
            vec!["--arg1".to_owned()],
            vec![("KEY".to_owned(), "VAL".to_owned())],
            MemorySettings::new(4096),
            WindowSettings::new(true, WindowSize::new(1920, 1080)),
            Hooks::new("pre".to_owned(), "wrap".to_owned(), "post".to_owned()),
        );

        assert_eq!(settings.launch_args(), &["--arg1"]);
        assert_eq!(settings.env_vars(), &[("KEY".to_owned(), "VAL".to_owned())]);
        assert_eq!(settings.memory().maximum, 4096);
        assert!(settings.window().force_fullscreen());
        assert_eq!(settings.window().game_resolution(), WindowSize(1920, 1080));
        assert_eq!(settings.hooks().pre_launch(), "pre");
        assert_eq!(settings.hooks().wrapper(), "wrap");
        assert_eq!(settings.hooks().post_exit(), "post");
    }

    #[test]
    fn should_set_launch_args_on_default_instance_settings() {
        let mut settings = DefaultInstanceSettings::default();

        settings.set_launch_args(vec!["--arg".to_owned()]);

        assert_eq!(settings.launch_args(), &["--arg"]);
    }

    #[test]
    fn should_set_env_vars_on_default_instance_settings() {
        let mut settings = DefaultInstanceSettings::default();

        settings.set_env_vars(vec![("A".to_owned(), "1".to_owned())]);

        assert_eq!(settings.env_vars(), &[("A".to_owned(), "1".to_owned())]);
    }

    #[test]
    fn should_set_memory_on_default_instance_settings() {
        let mut settings = DefaultInstanceSettings::default();

        settings.set_memory(MemorySettings::new(8192));

        assert_eq!(settings.memory().maximum, 8192);
    }

    #[test]
    fn should_set_window_on_default_instance_settings() {
        let mut settings = DefaultInstanceSettings::default();

        settings.set_window(WindowSettings::new(true, WindowSize::new(1280, 720)));

        assert!(settings.window().force_fullscreen());
        assert_eq!(settings.window().game_resolution(), WindowSize(1280, 720));
    }

    #[test]
    fn should_provide_mutable_hooks_reference() {
        let mut settings = DefaultInstanceSettings::default();

        settings
            .hooks_mut()
            .set_pre_launch("custom_hook".to_owned());

        assert_eq!(settings.hooks().pre_launch(), "custom_hook");
    }

    // ── MemorySettings ──

    #[test]
    fn should_create_memory_settings_with_default() {
        let memory = MemorySettings::default();

        assert_eq!(memory.maximum, 2048);
    }

    #[test]
    fn should_create_memory_settings_with_custom_value() {
        let memory = MemorySettings::new(4096);

        assert_eq!(memory.maximum, 4096);
    }

    // ── WindowSize ──

    #[test]
    fn should_create_window_size_with_default() {
        let size = WindowSize::default();

        assert_eq!(size.width(), 960);
        assert_eq!(size.height(), 540);
    }

    #[test]
    fn should_create_window_size_with_custom_dimensions() {
        let size = WindowSize::new(1920, 1080);

        assert_eq!(size.width(), 1920);
        assert_eq!(size.height(), 1080);
    }

    // ── WindowSettings ──

    #[test]
    fn should_create_window_settings_with_default() {
        let ws = WindowSettings::default();

        assert!(!ws.force_fullscreen());
        assert_eq!(ws.game_resolution(), WindowSize(960, 540));
    }

    #[test]
    fn should_construct_window_settings_with_new() {
        let ws = WindowSettings::new(true, WindowSize::new(1920, 1080));

        assert!(ws.force_fullscreen());
        assert_eq!(ws.game_resolution(), WindowSize(1920, 1080));
    }

    #[test]
    fn should_toggle_fullscreen_on_window_settings() {
        let mut ws = WindowSettings::default();

        ws.set_fullscreen(true);
        assert!(ws.force_fullscreen());

        ws.set_fullscreen(false);
        assert!(!ws.force_fullscreen());
    }

    #[test]
    fn should_set_resolution_on_window_settings() {
        let mut ws = WindowSettings::default();

        ws.set_resolution(WindowSize::new(640, 480));

        assert_eq!(ws.game_resolution(), WindowSize(640, 480));
    }

    // ── Hooks ──

    #[test]
    fn should_create_hooks_with_default() {
        let hooks = Hooks::default();

        assert!(hooks.pre_launch().is_empty());
        assert!(hooks.wrapper().is_empty());
        assert!(hooks.post_exit().is_empty());
    }

    #[test]
    fn should_construct_hooks_with_new() {
        let hooks = Hooks::new("a".to_owned(), "b".to_owned(), "c".to_owned());

        assert_eq!(hooks.pre_launch(), "a");
        assert_eq!(hooks.wrapper(), "b");
        assert_eq!(hooks.post_exit(), "c");
    }

    #[test]
    fn should_set_pre_launch_hook() {
        let mut hooks = Hooks::default();

        hooks.set_pre_launch("/path/to/pre.sh".to_owned());

        assert_eq!(hooks.pre_launch(), "/path/to/pre.sh");
    }

    #[test]
    fn should_set_wrapper_hook() {
        let mut hooks = Hooks::default();

        hooks.set_wrapper("/path/to/wrap.sh".to_owned());

        assert_eq!(hooks.wrapper(), "/path/to/wrap.sh");
    }

    #[test]
    fn should_set_post_exit_hook() {
        let mut hooks = Hooks::default();

        hooks.set_post_exit("/path/to/post.sh".to_owned());

        assert_eq!(hooks.post_exit(), "/path/to/post.sh");
    }
}
