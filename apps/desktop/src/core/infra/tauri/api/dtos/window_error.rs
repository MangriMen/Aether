use serde::Serialize;
use specta::Type;

use super::CauseDto;
use crate::core::{WindowError, WindowLabel};

#[derive(Clone, Debug, Serialize, Type)]
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
            WindowError::PlatformNotSupported => WindowErrorDto::PlatformNotSupported,
            // Tauri's own text: not ours to categorize.
            WindowError::Tauri(err) => WindowErrorDto::Other {
                cause: CauseDto::unknown(err),
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Type)]
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
    fn should_serialize_a_tauri_error_as_other_with_an_unknown_cause() {
        let dto = WindowErrorDto::from(WindowError::Tauri(tauri::Error::WebviewNotFound));

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({"code": "OTHER", "payload": {"cause": {"type": "unknown", "details": tauri::Error::WebviewNotFound.to_string()}}})
        );
    }

    #[test]
    fn should_serialize_platform_not_supported_without_payload() {
        let dto = WindowErrorDto::from(WindowError::PlatformNotSupported);

        assert_eq!(
            serde_json::to_value(&dto).unwrap(),
            json!({"code": "PLATFORM_NOT_SUPPORTED"})
        );
    }
}
