use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
    pub fraction: Option<f64>,
    pub phase: UpdatePhase,
    pub version: String,
    pub current_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePhase {
    Started,
    Progress,
    Finished,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_progress_fraction_none() {
        let progress = UpdateProgress {
            fraction: None,
            phase: UpdatePhase::Started,
            version: "1.0.0".to_string(),
            current_version: "0.9.0".to_string(),
        };
        assert!(progress.fraction.is_none());
    }

    #[test]
    fn update_progress_serialize_deserialize_roundtrip() {
        let progress = UpdateProgress {
            fraction: Some(0.75),
            phase: UpdatePhase::Finished,
            version: "2.0.0".to_string(),
            current_version: "1.0.0".to_string(),
        };
        let json = serde_json::to_string(&progress).unwrap();
        let deserialized: UpdateProgress = serde_json::from_str(&json).unwrap();
        assert!((deserialized.fraction.unwrap() - 0.75).abs() < f64::EPSILON);
        assert_eq!(deserialized.phase, UpdatePhase::Finished);
    }
}
