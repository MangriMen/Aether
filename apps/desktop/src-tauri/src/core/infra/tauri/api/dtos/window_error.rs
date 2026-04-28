use serde::Serialize;
use specta::Type;

use crate::core::{WindowError, WindowLabel};

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WindowErrorDto {
    NotFound { label: WindowLabelDto },

    PlatformNotSupported(String),

    Other(String),
}

impl From<WindowError> for WindowErrorDto {
    fn from(value: WindowError) -> Self {
        match value {
            WindowError::NotFound { label } => WindowErrorDto::NotFound {
                label: label.into(),
            },
            WindowError::PlatformNotSupported(err) => WindowErrorDto::PlatformNotSupported(err),
            WindowError::Tauri(err) => WindowErrorDto::Other(err.to_string()),
            WindowError::Other(err) => WindowErrorDto::Other(err.clone()),
        }
    }
}

#[derive(Debug, Serialize, Type)]
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
