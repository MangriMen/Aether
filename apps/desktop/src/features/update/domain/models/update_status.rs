use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateStatus {
    pub version: Option<String>,
    pub date: Option<DateTime<FixedOffset>>,
    pub body: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_status_default_all_none() {
        let status = UpdateStatus {
            version: None,
            date: None,
            body: None,
        };
        assert!(status.version.is_none());
        assert!(status.date.is_none());
        assert!(status.body.is_none());
    }

    #[test]
    fn update_status_with_values() {
        let status = UpdateStatus {
            version: Some("1.0.0".to_string()),
            date: None,
            body: Some("Bug fixes".to_string()),
        };
        assert_eq!(status.version.as_deref(), Some("1.0.0"));
        assert_eq!(status.body.as_deref(), Some("Bug fixes"));
    }
}
