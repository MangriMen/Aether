use serde::{Deserialize, Serialize};

use crate::features::update::UpdateProgress;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    Update(UpdateProgress),
}

impl From<UpdateProgress> for AppEvent {
    fn from(value: UpdateProgress) -> Self {
        Self::Update(value)
    }
}
