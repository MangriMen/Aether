use std::path::PathBuf;

use uuid::Uuid;

use crate::features::{
    auth::AuthApplicationError,
    instance::{ContentType, ProviderId},
    minecraft::MinecraftApplicationError,
    process::ProcessError,
    settings::SettingsError,
};

#[derive(Debug, thiserror::Error)]
pub enum InstanceError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Instance with id \"{instance_id}\" not found")]
    NotFound { instance_id: String },

    #[error(
        "Content provider not found for plugin \"{plugin_id}\" and capability \"{capability_id}\""
    )]
    ContentProviderNotFound {
        plugin_id: String,
        capability_id: String,
    },

    #[error("Capability operation error")]
    CapabilityOperationError,

    #[error("Instance \"{instance_id}\" still in installing state")]
    InstanceStillInstalling { instance_id: String },

    #[error("Instance \"{instance_id}\" already running with pid \"{process_id}\"")]
    InstanceAlreadyRunning {
        instance_id: String,
        process_id: Uuid,
    },

    #[error("Prelaunch command error with code: {code}")]
    PrelaunchCommandError { code: i32 },

    #[error("Instance validation error: \"{field:?}\" is invalid because \"{reason:?}\"")]
    ValidationError {
        field: InstanceField,
        reason: InstanceValidationErrorReason,
    },

    #[error("Not found importer {importer_id}")]
    ImporterNotFound { importer_id: String },

    #[error("Failed to import instance with {plugin_id} {capability_id}: {reason}")]
    ImportFailed {
        plugin_id: String,
        capability_id: String,
        /// Why the plugin's importer failed, as the core saw it.
        reason: String,
    },

    /// The plugin behind a capability was unloaded, or is restarting, when it was called.
    #[error(
        "Plugin \"{plugin_id}\" is not running: it was unloaded or is restarting. Try again once it is loaded"
    )]
    PluginNotRunning { plugin_id: String },

    // Update errors
    #[error("Not found pack info in instance")]
    PackInfoNotFound,

    #[error("Not found updater for modpack {modpack_id}")]
    UpdaterNotFound { modpack_id: String },

    #[error("Failed to update instance with modpack {modpack_id}")]
    UpdateFailed { modpack_id: String },

    #[error("Unmanaged instance")]
    UnmanagedInstance { instance_id: String },

    // Content
    #[error("Found duplicate content at {content_path}")]
    ContentDuplication { content_path: String },

    #[error("Can't get content file_name at path: {path}")]
    ContentFilename { path: PathBuf },

    // Instance files
    #[error("Path \"{path}\" is not a valid path inside an instance: {reason}")]
    InvalidRelativePath { path: String, reason: String },

    #[error("Path \"{path}\" resolves outside instance \"{instance_id}\"")]
    PathEscapesInstance { path: String, instance_id: String },

    #[error("File \"{path}\" is {size} bytes, over the {limit} byte limit")]
    FileTooLarge { path: String, size: u64, limit: u64 },

    #[error("Error when downloading content {0}")]
    ContentDownloadError(String),

    #[error("Content provider error {reason}")]
    ContentProviderError { reason: String },

    #[error("Not found content for minecraft version \"{game_version}\"")]
    ContentForGameVersionNotFound { game_version: String },

    #[error("Unsupported content type: {content_type:?}")]
    UnsupportedContentType { content_type: ContentType },

    /// A provider was asked for something it does not do — a single-mod provider for a pack
    /// preview, or one without install plans. The caller may treat it as "not offered"
    /// rather than as a failure.
    #[error("Content provider \"{provider_id}\" does not support {operation}")]
    UnsupportedOperation {
        provider_id: ProviderId,
        /// The provider method, as named in the trait (`resolve_pack_metadata`).
        operation: &'static str,
    },

    /// A [`ContentSource::ProviderRef`](super::ContentSource::ProviderRef) names a platform no
    /// available provider serves. There is no fallback to another platform's provider.
    #[error("No content provider serves platform \"{platform}\"")]
    PlatformProviderNotFound { platform: String },

    /// Several providers serve the platform and the core may not pick one itself: none is
    /// chosen in the settings and none is built in, or the chosen one is not available.
    #[error("Choose a content provider for platform \"{platform}\"")]
    PlatformProviderNotChosen { platform: String },

    /// A plan for an existing instance names a game to move it to, but is not a plan of the pack
    /// the instance was installed from: only an update of its own pack may change the game
    /// an instance runs.
    #[error(
        "Content \"{content_id}\" of \"{provider_id}\" may not change the game of instance \"{instance_id}\": it is not the pack the instance was installed from"
    )]
    GameChangeNotAllowed {
        instance_id: String,
        provider_id: ProviderId,
        content_id: String,
    },

    // Features errors
    #[error("Settings load error")]
    SettingsLoadError(#[from] SettingsError),

    #[error("Failed to get launch command")]
    MinecraftError(#[from] MinecraftApplicationError),

    #[error("Failed to launch instance")]
    ProcessError(#[from] ProcessError),

    #[error(transparent)]
    CredentialsError(#[from] AuthApplicationError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn should_format_storage_error_when_displayed() {
        let err = InstanceError::Storage("db error".into());
        assert_eq!(err.to_string(), "Storage error: db error");
    }

    #[test]
    fn should_format_not_found_error_when_displayed() {
        let err = InstanceError::NotFound {
            instance_id: "abc".into(),
        };
        assert_eq!(err.to_string(), "Instance with id \"abc\" not found");
    }

    #[test]
    fn should_format_capability_operation_error_when_displayed() {
        let err = InstanceError::CapabilityOperationError;
        assert_eq!(err.to_string(), "Capability operation error");
    }

    #[test]
    fn should_format_prelaunch_command_error_when_displayed() {
        let err = InstanceError::PrelaunchCommandError { code: 1 };
        assert_eq!(err.to_string(), "Prelaunch command error with code: 1");
    }

    #[test]
    fn should_format_unsupported_operation_error_when_displayed() {
        let err = InstanceError::UnsupportedOperation {
            provider_id: ProviderId {
                plugin_id: "aether".into(),
                capability_id: "modrinth".into(),
            },
            operation: "resolve_pack_metadata",
        };
        assert_eq!(
            err.to_string(),
            "Content provider \"aether#modrinth\" does not support resolve_pack_metadata"
        );
    }

    #[test]
    fn should_format_platform_provider_errors_when_displayed() {
        assert_eq!(
            InstanceError::PlatformProviderNotFound {
                platform: "curseforge".into()
            }
            .to_string(),
            "No content provider serves platform \"curseforge\""
        );
        assert_eq!(
            InstanceError::PlatformProviderNotChosen {
                platform: "curseforge".into()
            }
            .to_string(),
            "Choose a content provider for platform \"curseforge\""
        );
    }

    #[test]
    fn should_format_game_change_not_allowed_error_when_displayed() {
        let err = InstanceError::GameChangeNotAllowed {
            instance_id: "inst-1".into(),
            provider_id: ProviderId {
                plugin_id: "aether".into(),
                capability_id: "modrinth".into(),
            },
            content_id: "AANobbMI".into(),
        };
        assert_eq!(
            err.to_string(),
            "Content \"AANobbMI\" of \"aether#modrinth\" may not change the game of instance \"inst-1\": it is not the pack the instance was installed from"
        );
    }

    #[test]
    fn should_format_instance_already_running_error_when_displayed() {
        let pid = Uuid::new_v4();
        let err = InstanceError::InstanceAlreadyRunning {
            instance_id: "inst-1".into(),
            process_id: pid,
        };
        assert_eq!(
            err.to_string(),
            format!("Instance \"inst-1\" already running with pid \"{pid}\"",)
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceField {
    Name,
}

// impl InstanceField {
//     pub fn as_str(&self) -> &'static str {
//         match self {
//             InstanceField::Name => "name",
//         }
//     }
// }

// impl Display for InstanceField {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.as_str())
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceValidationErrorReason {
    CannotBeEmpty,
}

// impl ValidationErrorReason {
//     pub fn as_str(&self) -> &'static str {
//         match self {
//             ValidationErrorReason::CannotBeEmpty => "cannot_be_empty",
//         }
//     }
// }

// impl Display for ValidationErrorReason {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.as_str())
//     }
// }
