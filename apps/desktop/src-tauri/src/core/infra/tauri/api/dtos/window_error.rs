use serde::Serialize;
use specta::Type;

use crate::core::{WindowError, WindowLabel};

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WindowErrorDto {
    AlreadyExists { label: WindowLabelDto },

    NotFound { label: WindowLabelDto },

    PlatformNotSupported { details: String },

    Other { details: String },
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
            WindowError::PlatformNotSupported(err) => {
                WindowErrorDto::PlatformNotSupported { details: err }
            }
            WindowError::Tauri(err) => WindowErrorDto::Other {
                details: err.to_string(),
            },
            WindowError::Other(err) => WindowErrorDto::Other {
                details: err.clone(),
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
