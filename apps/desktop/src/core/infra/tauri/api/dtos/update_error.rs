use serde::Serialize;
use specta::Type;

use super::CauseDto;
use crate::features::update::UpdateError;

/// Why the launcher could not be updated.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UpdateErrorDto {
    NoPendingUpdate,
    GameRunning,
    Network { cause: CauseDto },
    Signature { cause: CauseDto },
    Install { cause: CauseDto },
    Other { cause: CauseDto },
}

impl From<UpdateError> for UpdateErrorDto {
    fn from(value: UpdateError) -> Self {
        let cause = |source: &aether_core::shared::error::SharedSource| {
            CauseDto::of_source(source.as_ref())
        };

        match value {
            UpdateError::NoPendingUpdate => Self::NoPendingUpdate,
            UpdateError::GameRunning => Self::GameRunning,
            UpdateError::Network(source) => Self::Network {
                cause: cause(&source),
            },
            UpdateError::Signature(source) => Self::Signature {
                cause: cause(&source),
            },
            UpdateError::Install(source) => Self::Install {
                cause: cause(&source),
            },
            UpdateError::Other(source) => Self::Other {
                cause: cause(&source),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use super::*;
    use crate::core::FrontendError;

    #[test]
    fn should_serialize_a_failed_download_with_the_category_of_its_cause() {
        let err = crate::Error::Update(UpdateError::Network(Arc::new(std::io::Error::from(
            std::io::ErrorKind::TimedOut,
        ))));

        assert_eq!(
            serde_json::to_value(FrontendError::from(err)).unwrap(),
            json!({"type": "update", "payload": {"code": "NETWORK", "payload": {"cause": {"type": "io", "kind": "timed_out"}}}})
        );
    }

    #[test]
    fn should_serialize_a_missing_pending_update_without_payload() {
        assert_eq!(
            serde_json::to_value(UpdateErrorDto::from(UpdateError::NoPendingUpdate)).unwrap(),
            json!({"code": "NO_PENDING_UPDATE"})
        );
    }

    #[test]
    fn should_serialize_a_running_game_when_it_has_no_payload() {
        assert_eq!(
            serde_json::to_value(UpdateErrorDto::from(UpdateError::GameRunning)).unwrap(),
            json!({"code": "GAME_RUNNING"})
        );
    }

    #[test]
    fn should_keep_the_text_of_an_uncategorized_updater_error_as_unknown() {
        let dto = UpdateErrorDto::from(UpdateError::Signature(Arc::new(std::io::Error::other(
            "bad signature",
        ))));

        assert_eq!(
            serde_json::to_value(dto).unwrap(),
            json!({"code": "SIGNATURE", "payload": {"cause": {"type": "unknown", "details": "bad signature"}}})
        );
    }
}
