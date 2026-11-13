use serde::Serialize;
use specta::Type;

use super::CauseDto;
use crate::core::{WindowError, WindowLabel};

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WindowErrorDto {
    AlreadyExists { label: WindowLabelDto },

    NotFound { label: WindowLabelDto },

    PlatformNotSupported,

    Other { cause: CauseDto },
}

impl From<WindowError> for WindowErrorDto {
    fn from(value: WindowError) -> Self {
        match value {
            WindowError::AlreadyExists { label } => WindowErrorDto::AlreadyExists {
                label: label.into(),
            },
            WindowError::NotFound { label } => WindowErrorDto::NotFound {
                label: label.into(),
            },
            // The core text is free-form and nothing constructs this variant; the log keeps it.
            WindowError::PlatformNotSupported(_) => WindowErrorDto::PlatformNotSupported,
            WindowError::Tauri(err) => WindowErrorDto::Other {
                cause: CauseDto::unknown(err),
            },
            WindowError::Other(err) => WindowErrorDto::Other {
                cause: CauseDto::unknown(err),
            },
        }
    }
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum WindowLabelDto {
    Main,
}

impl From<WindowLabel> for WindowLabelDto {
    fn from(value: WindowLabel) -> Self {
        match value {
            WindowLabel::Main => WindowLabelDto::Main,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn should_serialize_other_with_an_unknown_cause() {
        let dto = WindowErrorDto::from(WindowError::Other("boom".to_owned()));

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({"code": "OTHER", "payload": {"cause": {"type": "unknown", "details": "boom"}}})
        );
    }

    #[test]
    fn should_serialize_platform_not_supported_without_payload() {
        let dto = WindowErrorDto::from(WindowError::PlatformNotSupported("x".to_owned()));

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({"code": "PLATFORM_NOT_SUPPORTED"})
        );
    }
}
