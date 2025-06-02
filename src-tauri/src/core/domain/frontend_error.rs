use std::{collections::HashMap, error::Error};

pub type FrontendResult<T, E = FrontendError> = Result<T, E>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FrontendError {
    pub code: String,
    pub message: String,
    pub args: HashMap<String, String>,
}

impl From<crate::Error> for FrontendError {
    fn from(value: crate::Error) -> Self {
        FrontendError {
            code: "unknown".to_owned(),
            message: value.to_string(),
            args: HashMap::default(),
        }
    }
}

impl From<aether_core::Error> for FrontendError {
    fn from(value: aether_core::Error) -> Self {
        FrontendError {
            code: "unknown".to_owned(),
            message: value.to_string(),
            args: HashMap::default(),
        }
    }
}

impl From<Box<dyn Error + 'static>> for FrontendError {
    fn from(value: Box<dyn Error + 'static>) -> Self {
        FrontendError {
            code: "unknown".to_owned(),
            message: value.to_string(),
            args: HashMap::default(),
        }
    }
}

impl From<String> for FrontendError {
    fn from(value: String) -> Self {
        FrontendError {
            code: "unknown".to_owned(),
            message: value,
            args: HashMap::default(),
        }
    }
}

impl From<anyhow::Error> for FrontendError {
    fn from(value: anyhow::Error) -> Self {
        FrontendError {
            code: "unknown".to_owned(),
            message: value.to_string(),
            args: HashMap::default(),
        }
    }
}
