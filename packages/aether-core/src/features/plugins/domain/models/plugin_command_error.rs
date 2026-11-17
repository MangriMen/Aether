//! Why `run_command` failed once its command was allowed to run: typed all the way to the plugin,
//! so that a command that ran out of time reads as `timedOut`, not as the launcher's own failure.

/// A failure of running a command a plugin asked for. The refusals before it runs (a program or
/// a working directory the plugin may not use, the user's no) are [`PluginError`](super::PluginError)s.
#[derive(Debug, Clone, thiserror::Error)]
pub enum PluginCommandError {
    /// The process could not be started.
    #[error("Failed to run `{program}`: {reason}")]
    Spawn { program: String, reason: String },

    /// The process started, but one of its output streams could not be taken.
    #[error("Failed to capture command {stream}")]
    OutputNotCaptured { stream: &'static str },

    /// Reading what the process printed, or waiting for it, failed.
    #[error("Failed to read output of `{program}`: {reason}")]
    ReadOutput { program: String, reason: String },

    /// The process ran past its time and was killed, with everything it started.
    #[error("Command `{program}` exceeded the {seconds} second time limit and was killed")]
    TimedOut { program: String, seconds: u64 },
}
