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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::update::{UpdatePhase, UpdateProgress};

    fn make_progress() -> UpdateProgress {
        UpdateProgress {
            fraction: Some(0.5),
            phase: UpdatePhase::Progress,
            version: "1.0.0".to_string(),
            current_version: "0.9.0".to_string(),
        }
    }

    #[test]
    fn app_event_from_update_progress() {
        let progress = make_progress();
        let event: AppEvent = progress.clone().into();
        match event {
            AppEvent::Update(p) => assert_eq!(p.fraction, Some(0.5)),
        }
    }

    #[test]
    fn update_progress_try_from_app_event_ok() {
        let progress = make_progress();
        let event: AppEvent = progress.clone().into();
        let result: Result<UpdateProgress, _> = event.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version, "1.0.0");
    }

    #[test]
    fn app_event_serialize_deserialize_roundtrip() {
        let event: AppEvent = make_progress().into();
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AppEvent = serde_json::from_str(&json).unwrap();
        match deserialized {
            AppEvent::Update(p) => assert_eq!(p.fraction, Some(0.5)),
        }
    }
}
