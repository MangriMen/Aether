use serde::Serialize;
use specta::Type;

use aether_core::shared::io::IoErrorKind;

use super::CauseDto;
use crate::shared::IdempotencyManagerError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum IoErrorKindDto {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    StorageFull,
    ReadOnlyFilesystem,
    ResourceBusy,
    TimedOut,
    InvalidData,
}

impl IoErrorKindDto {
    /// `Other` has no frontend category: the caller reports it as `CauseDto::Unknown`.
    pub fn from_kind(value: IoErrorKind) -> Option<Self> {
        Some(match value {
            IoErrorKind::NotFound => Self::NotFound,
            IoErrorKind::PermissionDenied => Self::PermissionDenied,
            IoErrorKind::AlreadyExists => Self::AlreadyExists,
            IoErrorKind::StorageFull => Self::StorageFull,
            IoErrorKind::ReadOnlyFilesystem => Self::ReadOnlyFilesystem,
            IoErrorKind::ResourceBusy => Self::ResourceBusy,
            IoErrorKind::TimedOut => Self::TimedOut,
            IoErrorKind::InvalidData => Self::InvalidData,
            IoErrorKind::Other => return None,
        })
    }
}

#[derive(Clone, Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApplicationErrorDto {
    Io { cause: CauseDto },

    Internal { cause: CauseDto },

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
    use super::*;
    use crate::core::infra::tauri::api::dtos::FrontendError;
    use crate::shared::{IdempotencyManagerError, RequestId};
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case::other(std::io::Error::other("disk full"), json!({"type": "unknown", "details": "disk full"}))]
    #[case::not_found(
        std::io::Error::from(std::io::ErrorKind::NotFound),
        json!({"type": "io", "kind": "notFound"})
    )]
    #[case::storage_full(
        std::io::Error::from(std::io::ErrorKind::StorageFull),
        json!({"type": "io", "kind": "storageFull"})
    )]
    fn should_map_io_error_to_structured_application_error(
        #[case] io_error: std::io::Error,
        #[case] expected_cause: serde_json::Value,
    ) {
        let err = crate::Error::IoError(io_error);

        let value = serde_json::to_value(FrontendError::from(err)).unwrap();

        assert_eq!(value["type"], "application");
        assert_eq!(value["payload"]["code"], "IO");
        assert_eq!(value["payload"]["payload"]["cause"], expected_cause);
    }

    #[rstest]
    #[case::not_found(std::io::ErrorKind::NotFound, Some(IoErrorKindDto::NotFound))]
    #[case::permission(
        std::io::ErrorKind::PermissionDenied,
        Some(IoErrorKindDto::PermissionDenied)
    )]
    #[case::exists(std::io::ErrorKind::AlreadyExists, Some(IoErrorKindDto::AlreadyExists))]
    #[case::full(std::io::ErrorKind::StorageFull, Some(IoErrorKindDto::StorageFull))]
    #[case::quota(std::io::ErrorKind::QuotaExceeded, Some(IoErrorKindDto::StorageFull))]
    #[case::read_only(
        std::io::ErrorKind::ReadOnlyFilesystem,
        Some(IoErrorKindDto::ReadOnlyFilesystem)
    )]
    #[case::busy(std::io::ErrorKind::ResourceBusy, Some(IoErrorKindDto::ResourceBusy))]
    #[case::timed_out(std::io::ErrorKind::TimedOut, Some(IoErrorKindDto::TimedOut))]
    #[case::invalid(std::io::ErrorKind::InvalidData, Some(IoErrorKindDto::InvalidData))]
    #[case::other(std::io::ErrorKind::Interrupted, None)]
    fn should_map_std_io_kind_to_dto_kind(
        #[case] kind: std::io::ErrorKind,
        #[case] expected: Option<IoErrorKindDto>,
    ) {
        assert_eq!(IoErrorKindDto::from_kind(IoErrorKind::from(kind)), expected);
    }

    #[test]
    fn should_map_duplicate_request_to_structured_application_error() {
        let err = crate::Error::IdempotencyManager(IdempotencyManagerError::DuplicateRequest {
            id: RequestId::new("req-1".to_string()),
        });

        let value = serde_json::to_value(FrontendError::from(err)).unwrap();

        assert_eq!(value["payload"]["code"], "DUPLICATE_REQUEST");
        // The frontend looks the arguments up by this key: with none, it translates without any.
        assert!(value["payload"].get("payload").is_none());
    }

    #[test]
    fn should_map_a_launch_error_to_an_internal_error_when_it_reaches_the_frontend() {
        let err = crate::Error::LaunchError("port in use".to_string());

        let value = serde_json::to_value(FrontendError::from(err)).unwrap();

        assert_eq!(value["type"], "application");
        assert_eq!(value["payload"]["code"], "INTERNAL");
        assert_eq!(
            value["payload"]["payload"]["cause"],
            json!({"type": "unknown", "details": "port in use"})
        );
    }
}
