use std::{path::Path, sync::Arc};

use super::{CommandParseError, parse_command_line};

/// Which of the user's launch hooks a command is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookKind {
    PreLaunch,
    Wrapper,
    PostExit,
}

impl std::fmt::Display for HookKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::PreLaunch => "pre-launch",
            Self::Wrapper => "wrapper",
            Self::PostExit => "post-exit",
        })
    }
}

/// Why a hook command of the user could not be run.
///
/// Names the program only: the arguments may hold tokens, so they are kept out of the text.
#[derive(Debug, Clone, thiserror::Error)]
pub enum HookSpawnError {
    #[error("The {hook} command is invalid: {reason}")]
    Invalid {
        hook: HookKind,
        reason: CommandParseError,
    },

    #[error("The {hook} command \"{program}\" could not be started: {source}")]
    SpawnFailed {
        hook: HookKind,
        program: String,
        #[source]
        source: Arc<std::io::Error>,
    },

    #[error("The {hook} command \"{program}\" could not be waited for: {source}")]
    WaitFailed {
        hook: HookKind,
        program: String,
        #[source]
        source: Arc<std::io::Error>,
    },
}

impl HookSpawnError {
    pub fn spawn_failed(hook: HookKind, program: impl Into<String>, err: std::io::Error) -> Self {
        Self::SpawnFailed {
            hook,
            program: program.into(),
            source: Arc::new(err),
        }
    }
}

/// The program and the arguments of the wrapper command, or `None` when there is no wrapper.
///
/// A wrapper that is, as a whole, the path of a file is that program with no arguments: the
/// setting used to be read so, and a path with spaces in it was not quoted.
pub fn parse_wrapper(wrapper: &str) -> Result<Option<(String, Vec<String>)>, HookSpawnError> {
    let wrapper = wrapper.trim();
    if wrapper.is_empty() {
        return Ok(None);
    }
    if Path::new(wrapper).is_file() {
        return Ok(Some((wrapper.to_owned(), Vec::new())));
    }
    parse_command_line(wrapper)
        .map(Some)
        .map_err(|reason| HookSpawnError::Invalid {
            hook: HookKind::Wrapper,
            reason,
        })
}
