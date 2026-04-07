use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializableCommand {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
}

impl SerializableCommand {
    pub fn from_string(command: &str, current_dir: Option<&Path>) -> Result<Self, String> {
        let mut parts = command.split_whitespace();

        Ok(Self {
            program: parts.next().ok_or("Error to parse command")?.to_string(),
            args: parts.map(|s| s.to_string()).collect(),
            current_dir: current_dir.map(|current_dir| current_dir.to_path_buf()),
        })
    }

    pub fn from_command(command: &std::process::Command) -> Self {
        Self {
            program: command.get_program().to_string_lossy().to_string(),
            args: command
                .get_args()
                .map(|s| s.to_string_lossy().to_string())
                .collect(),
            current_dir: command.get_current_dir().map(|dir| dir.to_path_buf()),
        }
    }

    pub fn to_command(&self) -> std::process::Command {
        let mut command = std::process::Command::new(&self.program);
        if let Some(current_dir) = &self.current_dir {
            command.current_dir(current_dir);
        }
        command.args(&self.args);
        command
    }

    pub fn to_tokio_command(&self) -> tokio::process::Command {
        let mut command = tokio::process::Command::new(&self.program);
        if let Some(current_dir) = &self.current_dir {
            command.current_dir(current_dir);
        }
        command.args(&self.args);
        command
    }
}

impl FromStr for SerializableCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s, None)
    }
}
