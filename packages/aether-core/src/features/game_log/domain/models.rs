use crate::shared::io::domain::IoError;

pub const CONSOLE_PREFIX: &str = "console-";
const LOG_SUFFIX: &str = ".log";
const GZ_SUFFIX: &str = ".log.gz";

#[derive(Clone, Debug, thiserror::Error)]
pub enum GameLogError {
    #[error("Invalid log source {source_id}")]
    InvalidSource { source_id: String },

    #[error("Log source {source_id} not found")]
    SourceNotFound { source_id: String },

    #[error("Log {source_id} is larger than {limit} bytes unpacked")]
    TooLarge { source_id: String, limit: u64 },

    #[error(transparent)]
    Io(#[from] IoError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSourceKind {
    /// The game's output as the launcher captured it, one file per run.
    Console,
    /// A file of the game's own `logs` folder.
    Game,
}

/// Names a log file of an instance without a path: the kind picks the folder, the name is a
/// bare file name, so a source can never point outside the two log folders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogSourceRef {
    kind: LogSourceKind,
    name: String,
}

impl LogSourceRef {
    pub fn new(kind: LogSourceKind, name: impl Into<String>) -> Result<Self, GameLogError> {
        let name = name.into();
        if is_valid_name(kind, &name) {
            Ok(Self { kind, name })
        } else {
            let source_id = format!("{}:{name}", kind_tag(kind));
            Err(GameLogError::InvalidSource { source_id })
        }
    }

    pub fn parse(id: &str) -> Result<Self, GameLogError> {
        let invalid = || GameLogError::InvalidSource {
            source_id: id.to_string(),
        };
        let (tag, name) = id.split_once(':').ok_or_else(invalid)?;
        let kind = match tag {
            "console" => LogSourceKind::Console,
            "game" => LogSourceKind::Game,
            _ => return Err(invalid()),
        };
        Self::new(kind, name).map_err(|_| invalid())
    }

    pub fn id(&self) -> String {
        format!("{}:{}", kind_tag(self.kind), self.name)
    }

    pub fn kind(&self) -> LogSourceKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_compressed(&self) -> bool {
        self.name.ends_with(GZ_SUFFIX)
    }
}

fn kind_tag(kind: LogSourceKind) -> &'static str {
    match kind {
        LogSourceKind::Console => "console",
        LogSourceKind::Game => "game",
    }
}

fn is_valid_name(kind: LogSourceKind, name: &str) -> bool {
    let plain = !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(['/', '\\', ':', '\0'])
        && !name.chars().any(char::is_control);
    if !plain {
        return false;
    }
    match kind {
        LogSourceKind::Console => name.starts_with(CONSOLE_PREFIX) && name.ends_with(LOG_SUFFIX),
        LogSourceKind::Game => name.ends_with(LOG_SUFFIX) || name.ends_with(GZ_SUFFIX),
    }
}

#[derive(Debug, Clone)]
pub struct LogSource {
    pub source: LogSourceRef,
    /// Bytes on disk (packed for a `.gz`).
    pub size: u64,
    /// Unix milliseconds.
    pub modified: Option<i64>,
}

/// Where a page of the log starts, in bytes. Positions always sit on a line start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogCursor {
    /// The last lines of the file.
    Tail,
    /// The lines that end right before the position.
    Before(u64),
    /// The lines that start at the position.
    After(u64),
}

/// Whole lines of a log file and the byte range they were read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogPage {
    pub text: String,
    pub start: u64,
    pub end: u64,
    pub has_more_before: bool,
    pub has_more_after: bool,
    /// Bytes in the whole file when it was read.
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::console("console:console-20260101-000000-000.log")]
    #[case::game_log("game:latest.log")]
    #[case::game_gz("game:2026-11-10-1.log.gz")]
    fn should_accept_a_source_id_when_it_names_a_log_file(#[case] id: &str) {
        assert_eq!(LogSourceRef::parse(id).unwrap().id(), id);
    }

    #[rstest]
    #[case::traversal("game:../secret.log")]
    #[case::nested("game:a/b.log")]
    #[case::backslash("game:a\\b.log")]
    #[case::drive("game:C:evil.log")]
    #[case::not_a_log("game:options.txt")]
    #[case::console_prefix("console:latest.log")]
    #[case::no_kind("latest.log")]
    #[case::unknown_kind("crash:latest.log")]
    #[case::empty_name("game:")]
    fn should_reject_a_source_id_when_it_could_leave_the_log_folders(#[case] id: &str) {
        assert!(matches!(
            LogSourceRef::parse(id),
            Err(GameLogError::InvalidSource { .. })
        ));
    }
}
