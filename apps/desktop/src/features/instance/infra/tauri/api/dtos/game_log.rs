use aether_core::features::{
    events::GameLogEvent,
    game_log::{LogCursor, LogPage, LogSource, LogSourceKind},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Serialize, Deserialize, Clone, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct GameLogEventDto {
    /// The number of the event, from one counter for all the events the backend sends: the
    /// frontend drops what it has seen already.
    pub seq: u32,
    pub instance_id: String,
    pub source_id: String,
    pub start: u32,
    pub end: u32,
    /// Empty when the lines were too many to send; the range has to be read then.
    pub text: String,
}

impl From<GameLogEvent> for GameLogEventDto {
    fn from(value: GameLogEvent) -> Self {
        Self {
            seq: 0,
            instance_id: value.instance_id.to_string(),
            source_id: value.source_id,
            start: clamp(value.start),
            end: clamp(value.end),
            text: value.text.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Type)]
#[serde(rename_all = "snake_case")]
pub enum LogSourceKindDto {
    Console,
    Game,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct LogSourceDto {
    pub id: String,
    pub name: String,
    pub kind: LogSourceKindDto,
    pub size: u32,
    /// Unix milliseconds.
    #[specta(type = Option<f64>)]
    pub modified: Option<i64>,
}

impl From<LogSource> for LogSourceDto {
    fn from(value: LogSource) -> Self {
        Self {
            id: value.source.id(),
            name: value.source.name().to_string(),
            kind: match value.source.kind() {
                LogSourceKind::Console => LogSourceKindDto::Console,
                LogSourceKind::Game => LogSourceKindDto::Game,
            },
            size: clamp(value.size),
            modified: value.modified,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LogCursorDto {
    Tail,
    Before { at: u32 },
    After { at: u32 },
}

impl From<LogCursorDto> for LogCursor {
    fn from(value: LogCursorDto) -> Self {
        match value {
            LogCursorDto::Tail => Self::Tail,
            LogCursorDto::Before { at } => Self::Before(at.into()),
            LogCursorDto::After { at } => Self::After(at.into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct LogPageDto {
    pub text: String,
    pub start: u32,
    pub end: u32,
    pub has_more_before: bool,
    pub has_more_after: bool,
}

impl From<LogPage> for LogPageDto {
    fn from(value: LogPage) -> Self {
        Self {
            text: value.text,
            start: clamp(value.start),
            end: clamp(value.end),
            has_more_before: value.has_more_before,
            has_more_after: value.has_more_after,
        }
    }
}

/// Byte positions are at most a few hundred megabytes (the capture and unpacking are capped), and
/// `u64` cannot be exported to TypeScript, so they travel as `u32`.
fn clamp(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
