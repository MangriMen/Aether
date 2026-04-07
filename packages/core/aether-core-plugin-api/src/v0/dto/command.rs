use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandDto {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputDto {
    pub status: u32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl FromStr for CommandDto {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        Ok(Self {
            program: parts.next().ok_or("Failed to parse command")?.to_string(),
            args: parts.map(|s| s.to_string()).collect(),
            current_dir: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_command_from_str() {
        let cmd = CommandDto::from_str("git commit -m 'hello'").unwrap();
        assert_eq!(cmd.program, "git");
        assert_eq!(cmd.args, vec!["commit", "-m", "'hello'"]);
        assert!(cmd.current_dir.is_none());
    }

    #[test]
    fn test_command_serialization() {
        let cmd = CommandDto {
            program: "ls".to_string(),
            args: vec!["-la".to_string()],
            current_dir: None,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains(r#""program":"ls""#));
    }
}
