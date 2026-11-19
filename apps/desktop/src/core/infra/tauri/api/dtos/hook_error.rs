use aether_core::shared::serializable_command::{CommandParseError, HookKind, HookSpawnError};
use serde::Serialize;
use specta::Type;

use super::CauseDto;

/// Why a hook command of the user (pre-launch, wrapper, post-exit) could not be run.
#[derive(Clone, Debug, Serialize, Type)]
#[serde(
    tag = "code",
    content = "payload",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum HookErrorDto {
    Invalid {
        hook: HookKindDto,
        reason: CommandParseReasonDto,
    },
    SpawnFailed {
        hook: HookKindDto,
        program: String,
        cause: CauseDto,
    },
    WaitFailed {
        hook: HookKindDto,
        program: String,
        cause: CauseDto,
    },
}

/// Which of the user's launch hooks failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HookKindDto {
    PreLaunch,
    Wrapper,
    PostExit,
}

/// Why a text is not a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(
    tag = "code",
    content = "payload",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum CommandParseReasonDto {
    Empty,
    UnclosedQuote,
}

impl From<&HookSpawnError> for HookErrorDto {
    fn from(value: &HookSpawnError) -> Self {
        match value {
            HookSpawnError::Invalid { hook, reason } => Self::Invalid {
                hook: (*hook).into(),
                reason: (*reason).into(),
            },
            HookSpawnError::SpawnFailed {
                hook,
                program,
                source,
            } => Self::SpawnFailed {
                hook: (*hook).into(),
                program: program.clone(),
                cause: source.into(),
            },
            HookSpawnError::WaitFailed {
                hook,
                program,
                source,
            } => Self::WaitFailed {
                hook: (*hook).into(),
                program: program.clone(),
                cause: source.into(),
            },
        }
    }
}

impl From<HookKind> for HookKindDto {
    fn from(value: HookKind) -> Self {
        match value {
            HookKind::PreLaunch => Self::PreLaunch,
            HookKind::Wrapper => Self::Wrapper,
            HookKind::PostExit => Self::PostExit,
        }
    }
}

impl From<CommandParseError> for CommandParseReasonDto {
    fn from(value: CommandParseError) -> Self {
        match value {
            CommandParseError::Empty => Self::Empty,
            CommandParseError::UnclosedQuote => Self::UnclosedQuote,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn should_serialize_an_invalid_hook_with_its_kind_and_reason() {
        let err = HookSpawnError::Invalid {
            hook: HookKind::Wrapper,
            reason: CommandParseError::UnclosedQuote,
        };

        assert_eq!(
            serde_json::to_value(HookErrorDto::from(&err)).unwrap(),
            json!({"code": "INVALID", "payload": {"hook": "WRAPPER", "reason": {"code": "UNCLOSED_QUOTE"}}})
        );
    }

    #[test]
    fn should_serialize_a_hook_that_did_not_start_with_the_program_and_an_io_cause() {
        let err = HookSpawnError::spawn_failed(
            HookKind::PreLaunch,
            "x",
            std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        );

        assert_eq!(
            serde_json::to_value(HookErrorDto::from(&err)).unwrap(),
            json!({"code": "SPAWN_FAILED", "payload": {"hook": "PRE_LAUNCH", "program": "x", "cause": {"type": "io", "kind": "permission_denied"}}})
        );
    }
}
