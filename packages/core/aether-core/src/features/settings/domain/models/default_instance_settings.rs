#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

    pub fn update_launch_args(&mut self, launch_args: Vec<String>) -> bool {
        if self.launch_args != launch_args {
            self.launch_args = launch_args;
            return true;
        }
        false
    }

    pub fn update_env_vars(&mut self, env_vars: Vec<(String, String)>) -> bool {
        if self.env_vars != env_vars {
            self.env_vars = env_vars;
            return true;
        }
        false
    }

    pub fn update_memory(&mut self, memory: MemorySettings) -> bool {
        if self.memory != memory {
            self.memory = memory;
            return true;
        }
        false
    }

    pub fn update_window(&mut self, window: WindowSettings) -> bool {
        if self.window != window {
            self.window = window;
            return true;
        }
        false
    }

    pub fn update_hooks(
        &mut self,
        pre_launch: Option<String>,
        wrapper: Option<String>,
        post_exit: Option<String>,
    ) -> bool {
        self.hooks.update_hooks(pre_launch, wrapper, post_exit)
    }
}

/// Memory usage settings for Java.
///
/// Used to define the maximum amount of memory that can be allocated
/// to the JVM when launching a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn maximum(&self) -> u32 {
        self.maximum
    }
}

/// A 2D size, represented by a tuple of two integers
///
/// First is the width, second is the height of the window (width, height)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSize {
    width: u16,
    height: u16,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 960,
            height: 540,
        }
    }
}

impl WindowSize {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }
}

/// Represents the visual configuration of the game window.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

    pub fn update_hooks(
        &mut self,
        pre_launch: Option<String>,
        wrapper: Option<String>,
        post_exit: Option<String>,
    ) -> bool {
        let mut is_changed = false;

        if let Some(val) = pre_launch
            && self.pre_launch != val
        {
            self.pre_launch = val;
            is_changed = true;
        }

        if let Some(val) = wrapper
            && self.wrapper != val
        {
            self.wrapper = val;
            is_changed = true;
        }

        if let Some(val) = post_exit
            && self.post_exit != val
        {
            self.post_exit = val;
            is_changed = true;
        }

        is_changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Hooks::update_hooks ──

    #[test]
    fn should_update_all_hooks_and_return_true() {
        let mut hooks = Hooks::new("pre".into(), "wrap".into(), "post".into());
        assert!(hooks.update_hooks(
            Some("pre2".into()),
            Some("wrap2".into()),
            Some("post2".into()),
        ));
        assert_eq!(hooks.pre_launch(), "pre2");
        assert_eq!(hooks.wrapper(), "wrap2");
        assert_eq!(hooks.post_exit(), "post2");
    }

    #[test]
    fn should_return_false_when_nothing_changed() {
        let mut hooks = Hooks::new("pre".into(), "wrap".into(), "post".into());
        assert!(!hooks.update_hooks(Some("pre".into()), Some("wrap".into()), Some("post".into()),));
    }

    #[test]
    fn should_skip_none_fields_and_still_detect_change() {
        let mut hooks = Hooks::new("pre".into(), "wrap".into(), "post".into());
        assert!(hooks.update_hooks(Some("pre2".into()), None, None));
        assert_eq!(hooks.pre_launch(), "pre2");
        assert_eq!(hooks.wrapper(), "wrap");
    }

    #[test]
    fn should_return_true_when_only_partial_update() {
        let mut hooks = Hooks::new("pre".into(), "wrap".into(), "post".into());
        assert!(hooks.update_hooks(None, None, Some("post2".into())));
        assert_eq!(hooks.post_exit(), "post2");
    }

    #[test]
    fn should_not_change_field_when_none_passed() {
        let mut hooks = Hooks::new("pre".into(), "wrap".into(), "post".into());
        hooks.update_hooks(None, None, None);
        assert_eq!(hooks.pre_launch(), "pre");
        assert_eq!(hooks.wrapper(), "wrap");
        assert_eq!(hooks.post_exit(), "post");
    }

    // ── DefaultInstanceSettings::update_launch_args (representative) ──

    #[test]
    fn should_update_launch_args_and_return_true() {
        let mut settings = DefaultInstanceSettings::default();
        assert!(settings.update_launch_args(vec!["--arg".into()]));
        assert_eq!(settings.launch_args(), &["--arg"]);
    }

    #[test]
    fn should_return_false_when_launch_args_unchanged() {
        let mut settings = DefaultInstanceSettings::default();
        assert!(!settings.update_launch_args(vec![]));
    }

    #[test]
    fn should_update_hooks_on_default_instance_settings() {
        let mut settings = DefaultInstanceSettings::default();
        settings.update_hooks(Some("pre".into()), Some("wrap".into()), None);
        assert_eq!(settings.hooks().pre_launch(), "pre");
        assert_eq!(settings.hooks().wrapper(), "wrap");
    }
}
