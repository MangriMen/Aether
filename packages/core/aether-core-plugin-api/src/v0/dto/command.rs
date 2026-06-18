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

/// Parse a command string into parts, respecting single and double quotes.
/// Example: `git commit -m "my message"` → `program="git"`, `args=["commit", "-m", "my message"]`
fn split_shell_words(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    let mut chars = s.chars().peekable();
    for c in chars.by_ref() {
        match c {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

impl FromStr for CommandDto {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = split_shell_words(s);
        let mut parts = parts.into_iter();
        Ok(Self {
            program: parts.next().ok_or("Failed to parse command")?.clone(),
            args: parts.collect(),
            current_dir: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_command_from_str_simple() {
        let cmd = CommandDto::from_str("git commit -m hello").unwrap();
        assert_eq!(cmd.program, "git");
        assert_eq!(cmd.args, vec!["commit", "-m", "hello"]);
    }

    #[test]
    fn test_command_from_str_single_quotes() {
        let cmd = CommandDto::from_str("git commit -m 'hello world'").unwrap();
        assert_eq!(cmd.program, "git");
        assert_eq!(cmd.args, vec!["commit", "-m", "hello world"]);
    }

    #[test]
    fn test_command_from_str_double_quotes() {
        let cmd = CommandDto::from_str("git commit -m \"hello world\"").unwrap();
        assert_eq!(cmd.program, "git");
        assert_eq!(cmd.args, vec!["commit", "-m", "hello world"]);
    }

    #[test]
    fn test_command_from_str_mixed_quotes() {
        let cmd = CommandDto::from_str("echo \"double quoted\" 'single quoted'").unwrap();
        assert_eq!(cmd.program, "echo");
        assert_eq!(cmd.args, vec!["double quoted", "single quoted"]);
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
