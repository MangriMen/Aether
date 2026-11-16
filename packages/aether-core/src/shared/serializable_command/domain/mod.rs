mod hook;

pub use hook::{HookKind, HookSpawnError, Wrapper, parse_hook_command, parse_wrapper};

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableCommand {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
    /// Environment the child process is started with.
    ///
    /// `None` inherits the launcher's environment (user pre-launch and post-exit commands).
    /// `Some` passes exactly these variables (plugin `run_command`): the launcher's environment
    /// may carry tokens and credentials that a plugin-started process must not read.
    #[serde(default)]
    pub env: Option<BTreeMap<String, String>>,
}

impl SerializableCommand {
    /// Reads a command line typed by the user, the same way on every OS (see
    /// [`split_command_line`]). The first word is the program.
    pub fn from_string(
        command: &str,
        current_dir: Option<&Path>,
    ) -> Result<Self, CommandParseError> {
        let (program, args) = parse_command_line(command)?;

        Ok(Self {
            program,
            args,
            current_dir: current_dir.map(Path::to_path_buf),
            env: None,
        })
    }

    pub fn from_command(command: &std::process::Command) -> Self {
        Self {
            program: command.get_program().to_string_lossy().to_string(),
            args: command
                .get_args()
                .map(|s| s.to_string_lossy().to_string())
                .collect(),
            current_dir: command.get_current_dir().map(Path::to_path_buf),
            env: None,
        }
    }

    pub fn to_command(&self) -> std::process::Command {
        let mut command = std::process::Command::new(&self.program);
        if let Some(current_dir) = &self.current_dir {
            command.current_dir(current_dir);
        }
        if let Some(env) = &self.env {
            command.env_clear().envs(env);
        }
        command.args(&self.args);
        command
    }

    pub fn to_tokio_command(&self) -> tokio::process::Command {
        let mut command = tokio::process::Command::new(&self.program);
        if let Some(current_dir) = &self.current_dir {
            command.current_dir(current_dir);
        }
        if let Some(env) = &self.env {
            command.env_clear().envs(env);
        }
        command.args(&self.args);
        command
    }
}

impl FromStr for SerializableCommand {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s, None)
    }
}

/// Why a text is not a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CommandParseError {
    #[error("the command is empty")]
    Empty,

    #[error("a quote is not closed")]
    UnclosedQuote,
}

/// The program and the arguments of a command line typed by the user.
pub fn parse_command_line(text: &str) -> Result<(String, Vec<String>), CommandParseError> {
    let mut words = split_command_line(text)?.into_iter();
    match words.next() {
        Some(program) if !program.is_empty() => Ok((program, words.collect())),
        _ => Err(CommandParseError::Empty),
    }
}

/// Splits a command line into words, the same way on every OS and as the settings form does:
/// words are separated by whitespace; text inside `"..."` keeps its spaces and the quotes are
/// dropped (`-Dx="a b"` is `-Dx=a b`); `""` inside quotes is a literal quote. Backslashes and
/// single quotes are plain characters, so Windows paths need no escaping.
fn split_command_line(text: &str) -> Result<Vec<String>, CommandParseError> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut has_word = false;
    let mut chars = text.chars().peekable();

    while let Some(char) = chars.next() {
        if char == '"' {
            loop {
                match chars.next() {
                    None => return Err(CommandParseError::UnclosedQuote),
                    Some('"') if chars.peek() == Some(&'"') => {
                        chars.next();
                        current.push('"');
                    }
                    Some('"') => break,
                    Some(other) => current.push(other),
                }
            }
            has_word = true;
        } else if is_separator(char) {
            if has_word {
                words.push(std::mem::take(&mut current));
                has_word = false;
            }
        } else {
            current.push(char);
            has_word = true;
        }
    }

    if has_word {
        words.push(current);
    }

    Ok(words)
}

/// Whitespace as JavaScript's `\s` reads it, so a command splits as it does in the form.
fn is_separator(char: char) -> bool {
    (char.is_whitespace() && char != '\u{85}') || char == '\u{feff}'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct Vectors {
        vectors: Vec<Vector>,
    }

    #[derive(serde::Deserialize)]
    struct Vector {
        name: String,
        input: String,
        program: Option<String>,
        #[serde(default)]
        args: Vec<String>,
        error: Option<String>,
    }

    fn error_code(err: CommandParseError) -> &'static str {
        match err {
            CommandParseError::Empty => "EMPTY",
            CommandParseError::UnclosedQuote => "UNCLOSED_QUOTE",
        }
    }

    #[test]
    fn should_read_every_shared_command_line_vector_as_the_frontend_does() {
        let Vectors { vectors } =
            serde_json::from_str(include_str!("../../../../tests/fixtures/command-line.json"))
                .unwrap();
        assert!(!vectors.is_empty());

        for vector in vectors {
            let result = parse_command_line(&vector.input);
            match (&vector.error, result) {
                (Some(expected), Err(err)) => {
                    assert_eq!(error_code(err), expected, "{}", vector.name);
                }
                (None, Ok((program, args))) => {
                    assert_eq!(Some(program), vector.program, "{}", vector.name);
                    assert_eq!(args, vector.args, "{}", vector.name);
                }
                (expected, result) => {
                    panic!("{}: expected {expected:?}, got {result:?}", vector.name);
                }
            }
        }
    }

    #[test]
    fn should_give_the_working_directory_to_a_parsed_command() {
        let command =
            SerializableCommand::from_string("java -jar app.jar", Some(Path::new("dir"))).unwrap();

        assert_eq!(command.program, "java");
        assert_eq!(command.args, ["-jar", "app.jar"]);
        assert_eq!(command.current_dir.as_deref(), Some(Path::new("dir")));
    }
}
