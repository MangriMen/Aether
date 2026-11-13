use aether_core::features::process::ProcessError;
use serde::Serialize;
use specta::Type;

use super::CauseDto;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessErrorDto {
    KillError { id: String },
    WaitError { id: String },
    Io { cause: CauseDto },
}

impl From<&ProcessError> for ProcessErrorDto {
    fn from(value: &ProcessError) -> Self {
        match value {
            ProcessError::KillError { id } => Self::KillError { id: id.clone() },
            ProcessError::WaitError { id } => Self::WaitError { id: id.clone() },
            ProcessError::Io(err) => Self::Io { cause: err.into() },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::shared::io::IoError;
    use serde_json::json;

    #[test]
    fn should_serialize_io_with_an_io_cause() {
        let err = ProcessError::Io(IoError::IoError(std::io::Error::from(
            std::io::ErrorKind::PermissionDenied,
        )));

        assert_eq!(
            serde_json::to_value(ProcessErrorDto::from(&err)).unwrap(),
            json!({"code": "IO", "payload": {"cause": {"type": "io", "kind": "permissionDenied"}}})
        );
    }
}
