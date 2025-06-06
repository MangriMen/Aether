use std::{collections::HashMap, error::Error};

use crate::core::FrontendErrorKind;

pub type FrontendResult<T, E = FrontendError> = Result<T, E>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FrontendError {
    pub code: FrontendErrorKind,
    pub message: String,
    pub args: HashMap<String, String>,
}

impl From<crate::Error> for FrontendError {
    fn from(value: crate::Error) -> Self {
        FrontendError {
            code: FrontendErrorKind::Unknown,
            message: value.to_string(),
            args: HashMap::default(),
        }
    }
}

impl From<aether_core::Error> for FrontendError {
    fn from(value: aether_core::Error) -> Self {
        FrontendError {
            code: FrontendErrorKind::Unknown,
            message: value.to_string(),
            args: HashMap::default(),
        }
    }
}

impl From<Box<dyn Error + 'static>> for FrontendError {
    fn from(value: Box<dyn Error + 'static>) -> Self {
        FrontendError {
            code: FrontendErrorKind::Unknown,
            message: value.to_string(),
            args: HashMap::default(),
        }
    }
}

impl From<String> for FrontendError {
    fn from(value: String) -> Self {
        FrontendError {
            code: FrontendErrorKind::Unknown,
            message: value,
            args: HashMap::default(),
        }
    }
}

impl From<anyhow::Error> for FrontendError {
    fn from(value: anyhow::Error) -> Self {
        FrontendError {
            code: FrontendErrorKind::Unknown,
            message: value.to_string(),
            args: HashMap::default(),
        }
    }
}
