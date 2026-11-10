use serde::Serialize;
use specta::Type;

use crate::shared::IdempotencyManagerError;

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum IoErrorKindDto {
    NotFound,

    PermissionDenied,

    Other,
}

impl From<std::io::ErrorKind> for IoErrorKindDto {
    fn from(value: std::io::ErrorKind) -> Self {
        match value {
            std::io::ErrorKind::NotFound => Self::NotFound,
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApplicationErrorDto {
    Io {
        kind: IoErrorKindDto,
        details: String,
    },

    DuplicateRequest,
}

impl From<&IdempotencyManagerError> for ApplicationErrorDto {
    fn from(value: &IdempotencyManagerError) -> Self {
        match value {
            IdempotencyManagerError::DuplicateRequest { .. } => Self::DuplicateRequest,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::infra::tauri::api::dtos::FrontendError;
    use crate::shared::{IdempotencyManagerError, RequestId};
    use rstest::rstest;

    #[rstest]
    #[case::other(std::io::Error::other("disk full"), "other", Some("disk full"))]
    #[case::not_found(std::io::Error::from(std::io::ErrorKind::NotFound), "notFound", None)]
    fn should_map_io_error_to_structured_application_error(
        #[case] io_error: std::io::Error,
        #[case] expected_kind: &str,
        #[case] expected_details: Option<&str>,
    ) {
        let err = crate::Error::IoError(io_error);

        let value = serde_json::to_value(FrontendError::from(err)).unwrap();

        assert_eq!(value["type"], "application");
        assert_eq!(value["payload"]["code"], "IO");
        assert_eq!(value["payload"]["payload"]["kind"], expected_kind);
        if let Some(details) = expected_details {
            assert_eq!(value["payload"]["payload"]["details"], details);
        }
    }

    #[test]
    fn should_map_duplicate_request_to_structured_application_error() {
        let err = crate::Error::IdempotencyManager(IdempotencyManagerError::DuplicateRequest {
            id: RequestId::new("req-1".to_string()),
        });

        let value = serde_json::to_value(FrontendError::from(err)).unwrap();

        assert_eq!(value["payload"]["code"], "DUPLICATE_REQUEST");
    }

    #[test]
    fn should_map_a_launch_error_to_a_generic_error_when_it_reaches_the_frontend() {
        let err = crate::Error::LaunchError("port in use".to_string());

        let value = serde_json::to_value(FrontendError::from(err)).unwrap();

        assert_eq!(value["type"], "generic");
        assert_eq!(value["payload"], "port in use");
    }
}
