use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::update::UpdateStatus;

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct UpdateStatusDto {
    pub version: Option<String>,
    pub date: Option<String>,
    pub body: Option<String>,
}

impl From<UpdateStatus> for UpdateStatusDto {
    fn from(value: UpdateStatus) -> Self {
        Self {
            version: value.version,
            date: value.date,
            body: value.body,
        }
    }
}
