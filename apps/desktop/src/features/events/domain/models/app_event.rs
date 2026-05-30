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

impl TryFrom<AppEvent> for UpdateProgress {
    type Error = ();

    fn try_from(value: AppEvent) -> Result<Self, Self::Error> {
        match value {
            AppEvent::Update(progress) => Ok(progress),
            #[allow(unreachable_patterns)]
            _ => Err(()),
        }
    }
}
