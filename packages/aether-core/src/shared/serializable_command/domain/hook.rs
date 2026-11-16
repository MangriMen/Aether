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

/// The user's wrapper command, which starts java: its program and the arguments that go before
/// the java path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrapper {
    pub program: String,
    pub args: Vec<String>,
}

/// The wrapper command, or `None` when there is no wrapper.
///
/// A wrapper that is, as a whole, the path of a file is that program with no arguments: the
/// setting used to be read so, and a path with spaces in it was not quoted.
pub fn parse_wrapper(wrapper: &str) -> Result<Option<Wrapper>, HookSpawnError> {
    let wrapper = wrapper.trim();
    if wrapper.is_empty() {
        return Ok(None);
    }
    if Path::new(wrapper).is_file() {
        return Ok(Some(Wrapper {
            program: wrapper.to_owned(),
            args: Vec::new(),
        }));
    }
    let (program, args) =
        parse_command_line(wrapper).map_err(|reason| HookSpawnError::Invalid {
            hook: HookKind::Wrapper,
            reason,
        })?;
    Ok(Some(Wrapper { program, args }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_have_no_wrapper_when_it_is_blank() {
        assert_eq!(parse_wrapper("  \t ").unwrap(), None);
    }

    #[test]
    fn should_split_the_wrapper_into_program_and_arguments() {
        assert_eq!(
            parse_wrapper(" env VAR=1 \"prime run\" ").unwrap(),
            Some(Wrapper {
                program: "env".to_owned(),
                args: vec!["VAR=1".to_owned(), "prime run".to_owned()],
            })
        );
    }

    #[test]
    fn should_take_the_whole_wrapper_as_the_program_when_it_is_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("my wrapper.sh");
        std::fs::write(&path, "").unwrap();
        let path = path.to_string_lossy().into_owned();

        assert_eq!(
            parse_wrapper(&format!(" {path} ")).unwrap(),
            Some(Wrapper {
                program: path,
                args: Vec::new(),
            })
        );
    }

    #[test]
    fn should_refuse_a_wrapper_with_a_quote_that_is_not_closed() {
        let result = parse_wrapper("\"prime-run --x");

        assert!(
            matches!(
                result,
                Err(HookSpawnError::Invalid {
                    hook: HookKind::Wrapper,
                    reason: CommandParseError::UnclosedQuote,
                })
            ),
            "{result:?}"
        );
    }
}
