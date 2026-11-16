use std::{path::Path, sync::Arc};

use super::{CommandParseError, SerializableCommand, is_separator, parse_command_line};

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

/// Whether a hook command has no words, so the hook is not set. Read with the parser's own idea
/// of whitespace, not [`str::trim`]: the two differ on U+FEFF and U+0085.
fn is_unset(command: &str) -> bool {
    command.chars().all(is_separator)
}

/// The user's `hook` command, started in `dir`, or `None` when the hook is not set.
pub fn parse_hook_command(
    hook: HookKind,
    command: &str,
    dir: &Path,
) -> Result<Option<SerializableCommand>, HookSpawnError> {
    if is_unset(command) {
        return Ok(None);
    }
    SerializableCommand::from_string(command, Some(dir))
        .map(Some)
        .map_err(|reason| HookSpawnError::Invalid { hook, reason })
}

/// The wrapper command, or `None` when there is no wrapper.
///
/// A wrapper that is, as a whole, the path of a file is that program with no arguments: the
/// setting used to be read so, and a path with spaces in it was not quoted. The path is looked up
/// as the game is started: from `launch_dir`.
pub fn parse_wrapper(wrapper: &str, launch_dir: &Path) -> Result<Option<Wrapper>, HookSpawnError> {
    if is_unset(wrapper) {
        return Ok(None);
    }
    let wrapper = wrapper.trim_matches(is_separator);
    if is_existing_program(&launch_dir.join(wrapper)) {
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

/// Whether `path` is a file a process can be started from. On Windows a path without an
/// extension is also started as `.exe`.
fn is_existing_program(path: &Path) -> bool {
    path.is_file()
        || (cfg!(windows) && path.extension().is_none() && path.with_extension("exe").is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn here() -> &'static Path {
        Path::new(".")
    }

    #[test]
    fn should_have_no_wrapper_when_it_is_blank() {
        for wrapper in ["", "  \t ", "\u{feff}"] {
            assert_eq!(parse_wrapper(wrapper, here()).unwrap(), None, "{wrapper:?}");
        }
    }

    #[test]
    fn should_split_the_wrapper_into_program_and_arguments() {
        assert_eq!(
            parse_wrapper(" env VAR=1 \"prime run\" ", here()).unwrap(),
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
            parse_wrapper(&format!(" {path} "), here()).unwrap(),
            Some(Wrapper {
                program: path,
                args: Vec::new(),
            })
        );
    }

    #[test]
    fn should_look_for_a_relative_wrapper_file_in_the_launch_folder() {
        let launch_dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(launch_dir.path().join("my tools")).unwrap();
        std::fs::write(launch_dir.path().join("my tools").join("my wrapper.sh"), "").unwrap();

        assert_eq!(
            parse_wrapper("my tools/my wrapper.sh", launch_dir.path()).unwrap(),
            Some(Wrapper {
                program: "my tools/my wrapper.sh".to_owned(),
                args: Vec::new(),
            })
        );
    }

    #[cfg(windows)]
    #[test]
    fn should_take_a_wrapper_file_named_without_exe_as_windows_starts_it() {
        let launch_dir = tempfile::tempdir().unwrap();
        std::fs::write(launch_dir.path().join("my wrapper.exe"), "").unwrap();

        assert_eq!(
            parse_wrapper("my wrapper", launch_dir.path()).unwrap(),
            Some(Wrapper {
                program: "my wrapper".to_owned(),
                args: Vec::new(),
            })
        );
    }

    #[test]
    fn should_refuse_a_wrapper_with_a_quote_that_is_not_closed() {
        let result = parse_wrapper("\"prime-run --x", here());

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

    #[test]
    fn should_have_no_hook_command_when_it_has_no_words() {
        for command in ["", " \t\r\n", "\u{feff}", "\u{a0}\u{feff} "] {
            let result = parse_hook_command(HookKind::PostExit, command, here());
            assert!(matches!(result, Ok(None)), "{command:?}: {result:?}");
        }
    }

    #[test]
    fn should_refuse_a_hook_command_whose_program_is_empty() {
        for command in ["\"\"", "\"\" arg"] {
            let result = parse_hook_command(HookKind::PreLaunch, command, here());
            assert!(
                matches!(
                    result,
                    Err(HookSpawnError::Invalid {
                        hook: HookKind::PreLaunch,
                        reason: CommandParseError::Empty,
                    })
                ),
                "{command:?}: {result:?}"
            );
        }
    }

    #[test]
    fn should_run_a_next_line_character_as_a_program_as_the_form_reads_it() {
        let command = parse_hook_command(HookKind::PostExit, "\u{85}", here())
            .unwrap()
            .unwrap();

        assert_eq!(command.program, "\u{85}");
        assert_eq!(command.current_dir.as_deref(), Some(here()));
    }
}
