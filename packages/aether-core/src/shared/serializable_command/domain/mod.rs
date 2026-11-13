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
    pub fn from_string(
        command: &str,
        current_dir: Option<&Path>,
    ) -> Result<Self, CommandParseError> {
        let mut parts = command.split_whitespace();

        Ok(Self {
            program: parts.next().ok_or(CommandParseError::Empty)?.to_string(),
            args: parts.map(ToString::to_string).collect(),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_refuse_an_empty_command() {
        assert_eq!(
            SerializableCommand::from_string("   ", None).unwrap_err(),
            CommandParseError::Empty
        );
    }

    #[test]
    fn should_split_a_command_into_program_and_arguments() {
        let command = SerializableCommand::from_string("java -jar app.jar", None).unwrap();

        assert_eq!(command.program, "java");
        assert_eq!(command.args, ["-jar", "app.jar"]);
    }
}
