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
    use rstest::rstest;

    // ── Hooks::update_hooks ──

    #[rstest]
    #[case::all_differ(
        (Some("pre2"), Some("wrap2"), Some("post2")),
        true,
        ("pre2", "wrap2", "post2")
    )]
    #[case::nothing_changed(
        (Some("pre"), Some("wrap"), Some("post")),
        false,
        ("pre", "wrap", "post")
    )]
    #[case::only_pre_given((Some("pre2"), None, None), true, ("pre2", "wrap", "post"))]
    #[case::only_post_given((None, None, Some("post2")), true, ("pre", "wrap", "post2"))]
    #[case::none_given((None, None, None), false, ("pre", "wrap", "post"))]
    fn should_update_the_given_hooks_and_report_whether_they_changed(
        #[case] update: (Option<&str>, Option<&str>, Option<&str>),
        #[case] expected_changed: bool,
        #[case] expected: (&str, &str, &str),
    ) {
        let mut hooks = Hooks::new("pre".into(), "wrap".into(), "post".into());
        let changed = hooks.update_hooks(
            update.0.map(Into::into),
            update.1.map(Into::into),
            update.2.map(Into::into),
        );
        assert_eq!(changed, expected_changed);
        assert_eq!(hooks.pre_launch(), expected.0);
        assert_eq!(hooks.wrapper(), expected.1);
        assert_eq!(hooks.post_exit(), expected.2);
    }

    // ── DefaultInstanceSettings::update_launch_args (representative) ──

    #[test]
    fn should_update_launch_args_and_return_true_when_they_differ() {
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
    fn should_update_hooks_when_given_default_instance_settings() {
        let mut settings = DefaultInstanceSettings::default();
        settings.update_hooks(Some("pre".into()), Some("wrap".into()), None);
        assert_eq!(settings.hooks().pre_launch(), "pre");
        assert_eq!(settings.hooks().wrapper(), "wrap");
    }
}
