use aether_core::features::instance::{BrokenInstance, BrokenInstanceOrigin, BrokenInstanceReason};
use serde::Serialize;
use specta::Type;

#[derive(Serialize, Clone, Copy, Debug, Eq, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum BrokenInstanceOriginDto {
    /// A record of the database that cannot be read.
    Database,
    /// A folder with a legacy instance file that is not in the database.
    LegacyFile,
}

/// Why an instance cannot be read; the code picks the translation.
#[derive(Serialize, Clone, Debug, Eq, PartialEq, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BrokenInstanceReasonDto {
    InvalidId,
    InvalidGameVersion { value: String },
    InvalidLoader { value: String },
    InvalidInstallStage { value: String },
    InvalidColumn { column: String },
    UnreadableFile { details: String },
    NotMigrated,
}

/// An instance the launcher stores but cannot read.
#[derive(Serialize, Clone, Debug, Eq, PartialEq, Type)]
#[serde(rename_all = "camelCase")]
pub struct BrokenInstanceDto {
    /// The id as stored; not necessarily one that names a folder.
    pub id: String,
    /// Its folder, for an id that names one.
    pub dir: Option<String>,
    pub origin: BrokenInstanceOriginDto,
    pub reason: BrokenInstanceReasonDto,
}

impl From<BrokenInstance> for BrokenInstanceDto {
    fn from(value: BrokenInstance) -> Self {
        Self {
            id: value.id,
            dir: value.dir.map(|dir| dir.to_string_lossy().into_owned()),
            origin: match value.origin {
                BrokenInstanceOrigin::Database => BrokenInstanceOriginDto::Database,
                BrokenInstanceOrigin::LegacyFile => BrokenInstanceOriginDto::LegacyFile,
            },
            reason: value.reason.into(),
        }
    }
}

impl From<BrokenInstanceReason> for BrokenInstanceReasonDto {
    fn from(value: BrokenInstanceReason) -> Self {
        match value {
            BrokenInstanceReason::InvalidId => Self::InvalidId,
            BrokenInstanceReason::InvalidGameVersion { value } => {
                Self::InvalidGameVersion { value }
            }
            BrokenInstanceReason::InvalidLoader { value } => Self::InvalidLoader { value },
            BrokenInstanceReason::InvalidInstallStage { value } => {
                Self::InvalidInstallStage { value }
            }
            BrokenInstanceReason::InvalidColumn { column } => Self::InvalidColumn { column },
            BrokenInstanceReason::UnreadableFile { details } => Self::UnreadableFile { details },
            BrokenInstanceReason::NotMigrated => Self::NotMigrated,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn should_serialize_the_reason_by_code_with_its_payload() {
        let dto = BrokenInstanceDto::from(BrokenInstance {
            id: "a".to_owned(),
            dir: Some(PathBuf::from("instances/a")),
            origin: BrokenInstanceOrigin::Database,
            reason: BrokenInstanceReason::InvalidGameVersion {
                value: "x".to_owned(),
            },
        });

        let value = serde_json::to_value(dto).unwrap();

        assert_eq!(value["origin"], "database");
        assert_eq!(value["reason"]["code"], "INVALID_GAME_VERSION");
        assert_eq!(value["reason"]["payload"]["value"], "x");
    }

    #[test]
    fn should_send_no_folder_when_the_id_names_none() {
        let dto = BrokenInstanceDto::from(BrokenInstance {
            id: "../x".to_owned(),
            dir: None,
            origin: BrokenInstanceOrigin::Database,
            reason: BrokenInstanceReason::InvalidId,
        });

        let value = serde_json::to_value(dto).unwrap();

        assert!(value["dir"].is_null());
        assert_eq!(value["reason"], serde_json::json!({ "code": "INVALID_ID" }));
    }
}
