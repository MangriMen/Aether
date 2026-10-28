use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandDto {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
}

/// Output of a command run through `run_command`. The streams travel as msgpack `bin`.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputDto {
    pub status: u32,
    #[serde_as(as = "Bytes")]
    pub stdout: Vec<u8>,
    #[serde_as(as = "Bytes")]
    pub stderr: Vec<u8>,
    /// `stdout` hit the host's per-stream cap and was cut short.
    // `default` lets a plugin built against this DTO still decode a response from a host that
    // predates the flags. The other way round does not work: the ABI encodes DTOs as
    // fixed-length msgpack arrays, so a plugin built before the flags rejects a response that
    // carries them, so the core is released before the plugins.
    #[serde(default)]
    pub stdout_truncated: bool,
    /// `stderr` hit the host's per-stream cap and was cut short.
    #[serde(default)]
    pub stderr_truncated: bool,
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
    use rstest::rstest;
    use std::str::FromStr;

    #[rstest]
    #[case::simple("git commit -m hello", "git", &["commit", "-m", "hello"])]
    #[case::single_quotes("git commit -m 'hello world'", "git", &["commit", "-m", "hello world"])]
    #[case::double_quotes("git commit -m \"hello world\"", "git", &["commit", "-m", "hello world"])]
    #[case::mixed_quotes(
        "echo \"double quoted\" 'single quoted'",
        "echo",
        &["double quoted", "single quoted"]
    )]
    fn should_split_the_program_and_args_when_parsing_a_command_line(
        #[case] line: &str,
        #[case] program: &str,
        #[case] args: &[&str],
    ) {
        let cmd = CommandDto::from_str(line).unwrap();
        assert_eq!(cmd.program, program);
        assert_eq!(cmd.args, args);
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
