use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApplicationErrorDto {
    Io { details: String },

    DuplicateRequest { id: String },

    Launch { details: String },
}

impl From<&crate::shared::IdempotencyManagerError> for ApplicationErrorDto {
    fn from(value: &crate::shared::IdempotencyManagerError) -> Self {
        match value {
            crate::shared::IdempotencyManagerError::DuplicateRequest { id } => {
                Self::DuplicateRequest { id: id.to_string() }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::infra::tauri::api::dtos::FrontendError;
    use crate::shared::{IdempotencyManagerError, RequestId};

    #[test]
    fn should_map_io_error_to_structured_application_error() {
        let err = crate::Error::IoError(std::io::Error::other("disk full"));

        let value = serde_json::to_value(FrontendError::from(err)).unwrap();

        assert_eq!(value["type"], "application");
        assert_eq!(value["payload"]["code"], "IO");
        assert_eq!(value["payload"]["payload"]["details"], "disk full");
    }

    #[test]
    fn should_map_duplicate_request_to_structured_application_error() {
        let err = crate::Error::IdempotencyManager(IdempotencyManagerError::DuplicateRequest {
            id: RequestId::new("req-1".to_string()),
        });

        let value = serde_json::to_value(FrontendError::from(err)).unwrap();

        assert_eq!(value["payload"]["code"], "DUPLICATE_REQUEST");
        assert_eq!(value["payload"]["payload"]["id"], "req-1");
    }

    #[test]
    fn should_map_launch_error_to_application_error_instead_of_internal() {
        let err = crate::Error::LaunchError("port in use".to_string());

        let value = serde_json::to_value(FrontendError::from(err)).unwrap();

        assert_eq!(value["type"], "application");
        assert_eq!(value["payload"]["code"], "LAUNCH");
    }
}
