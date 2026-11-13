use aether_core::features::instance::ProviderIdError;
use aether_core::features::settings::SettingsError;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SettingsErrorDto {
    NotFound,

    StorageFailure,

    InvalidPlatformProvider {
        platform: String,
        reason: ProviderIdReasonDto,
    },

    InvalidMaxConcurrentDownloads,

    InvalidMemory,
}

/// Why a text is not a `plugin#capability` provider id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderIdReasonDto {
    /// The text is not two parts around the separator.
    Format { value: String },
    /// A part is outside the id scheme. Which one is not told: the value names it.
    InvalidPart { value: String },
}

impl From<&ProviderIdError> for ProviderIdReasonDto {
    fn from(value: &ProviderIdError) -> Self {
        match value {
            ProviderIdError::Format(text) => Self::Format {
                value: text.clone(),
            },
            ProviderIdError::InvalidPart(part) => Self::InvalidPart {
                value: part.value.clone(),
            },
        }
    }
}

impl From<&SettingsError> for SettingsErrorDto {
    fn from(value: &SettingsError) -> Self {
        match value {
            SettingsError::NotFound => SettingsErrorDto::NotFound,
            SettingsError::Storage(_) => SettingsErrorDto::StorageFailure,
            SettingsError::InvalidPlatformProvider { platform, reason } => {
                SettingsErrorDto::InvalidPlatformProvider {
                    platform: platform.clone(),
                    reason: reason.into(),
                }
            }
            SettingsError::InvalidMaxConcurrentDownloads => {
                SettingsErrorDto::InvalidMaxConcurrentDownloads
            }
            SettingsError::InvalidMemory => SettingsErrorDto::InvalidMemory,
        }
    }
}

#[cfg(test)]
mod tests {
    use aether_core::features::instance::InvalidId;
    use serde_json::json;

    use super::*;

    fn to_json(err: &SettingsError) -> serde_json::Value {
        serde_json::to_value(SettingsErrorDto::from(err)).unwrap()
    }

    #[test]
    fn should_serialize_a_malformed_provider_id_as_a_format_reason() {
        let err = SettingsError::InvalidPlatformProvider {
            platform: "curseforge".to_owned(),
            reason: ProviderIdError::Format("nope".to_owned()),
        };

        assert_eq!(
            to_json(&err),
            json!({
                "code": "INVALID_PLATFORM_PROVIDER",
                "payload": {
                    "platform": "curseforge",
                    "reason": { "code": "FORMAT", "payload": { "value": "nope" } },
                },
            })
        );
    }

    #[test]
    fn should_serialize_a_part_outside_the_id_scheme_as_an_invalid_part_reason() {
        let err = SettingsError::InvalidPlatformProvider {
            platform: "curseforge".to_owned(),
            reason: ProviderIdError::InvalidPart(InvalidId {
                kind: "plugin",
                value: "Bad Id".to_owned(),
            }),
        };

        assert_eq!(
            to_json(&err),
            json!({
                "code": "INVALID_PLATFORM_PROVIDER",
                "payload": {
                    "platform": "curseforge",
                    "reason": {
                        "code": "INVALID_PART",
                        "payload": { "value": "Bad Id" },
                    },
                },
            })
        );
    }

    #[test]
    fn should_serialize_the_variants_without_arguments_as_a_bare_code() {
        assert_eq!(
            to_json(&SettingsError::InvalidMemory),
            json!({"code": "INVALID_MEMORY"})
        );
    }
}
