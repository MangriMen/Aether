use std::path::PathBuf;

use aether_core::features::plugins::{
    AllowedHostEntryRejection, AllowedHostRejection, AllowedPath, AllowedPathInput,
    AllowedPathRejection, CommandIdentity, EditPluginSettings, FolderAccess, HostAccess,
    IgnoredAllowedPath, PluginSettings,
};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Default, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginSettingsDto {
    pub allowed_hosts: Vec<String>,

    /// Local hosts (`host` or `host:port`) the user let the plugin reach at a non-public
    /// address, to take back from the settings screen. Granted only by
    /// `grant_local_host_access`, never by an edit.
    pub local_hosts: Vec<String>,

    pub allowed_paths: Vec<AllowedPathDto>,

    /// Saved folders that were left out because they are not valid (for example the plugin side
    /// is not a path a plugin may have). Never mounted, but kept in the file until the user
    /// removes them: send the ones to keep as `ignoredPaths` of an edit.
    pub ignored_paths: Vec<IgnoredAllowedPathDto>,

    /// Programs the user let the plugin run for good, to take back from the settings screen.
    pub command_approvals: Vec<CommandApprovalDto>,
}

#[derive(Serialize, Deserialize, Debug, Default, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditPluginSettingsDto {
    #[specta(optional)]
    pub allowed_hosts: Option<Vec<String>>,
    #[specta(optional)]
    pub allowed_paths: Option<Vec<AllowedPathDto>>,
    /// The ignored folders to keep; every other one is deleted. Omit to keep them all.
    #[specta(optional)]
    pub ignored_paths: Option<Vec<AllowedPathDto>>,
    /// The approvals to take back. Never adds one.
    #[specta(optional)]
    pub revoke_command_approvals: Option<Vec<CommandApprovalKeyDto>>,
    /// The local hosts to take back, as `localHosts` lists them. Never adds one.
    #[specta(optional)]
    pub revoke_local_hosts: Option<Vec<String>>,
}

/// A program the user let a plugin run for good.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct CommandApprovalDto {
    #[serde(flatten)]
    pub key: CommandApprovalKeyDto,
    /// The program as the plugin names it (`/cache/...`).
    pub jar: String,
    /// When the user said yes, RFC 3339.
    pub approved_at: String,
}

/// What an approval is told apart by.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct CommandApprovalKeyDto {
    pub jar_sha256: String,
    pub jvm_options: Vec<String>,
}

/// A folder of the user's given to a plugin.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct AllowedPathDto {
    /// Path on the host.
    pub host_path: String,
    /// Path inside the plugin sandbox. A host-style path is turned into the plugin's name for
    /// it (`D:\packs` → `/mnt/d/packs`).
    pub plugin_path: String,
    /// `true` unless the user lets the plugin write there.
    pub read_only: bool,
}

/// A saved folder that was not read, and why.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct IgnoredAllowedPathDto {
    pub path: AllowedPathDto,
    pub reason: AllowedPathRejectionDto,
}

#[derive(Serialize, Debug, Type)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum FolderAccessDto {
    /// The plugin could already read the file.
    AlreadyAllowed { folder: PathBuf },
    /// A dry run: the plugin cannot read the file yet; nothing was changed.
    Needed { folder: PathBuf },
    /// The folder was added read-only and the plugin restarted with it.
    Granted { folder: PathBuf },
}

#[derive(Serialize, Debug, Type)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum HostAccessDto {
    /// The plugin could already reach the host.
    AlreadyAllowed { host: String },
    /// A dry run: the plugin cannot reach the host yet; nothing was changed.
    Needed { host: String },
    /// The host was added to the plugin's allowed hosts.
    Granted { host: String },
}

#[derive(Serialize, Debug, Clone, Copy, Type)]
#[serde(rename_all = "snake_case")]
pub enum AllowedHostEntryRejectionDto {
    Empty,
    NotAHostName,
    InvalidPattern,
}

#[derive(Serialize, Debug, Clone, Copy, Type)]
#[serde(rename_all = "snake_case")]
pub enum AllowedHostRejectionDto {
    InvalidUrl,
    UnsupportedScheme,
    MissingHost,
    NotPublic,
    NotLocal,
    Unresolved,
    Wildcard,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Type)]
#[serde(rename_all = "snake_case")]
pub enum AllowedPathRejectionDto {
    HostNotAbsolute,
    HostNotAFolder,
    HostIsRoot,
    HostIsHome,
    PluginNotAbsolute,
    PluginNotNormalized,
    PluginIsRoot,
    PluginReserved,
    HostOverlapsCore,
}

impl From<PluginSettings> for PluginSettingsDto {
    fn from(value: PluginSettings) -> Self {
        Self {
            allowed_hosts: value.allowed_hosts,
            local_hosts: value.local_hosts.iter().map(ToString::to_string).collect(),
            allowed_paths: value.allowed_paths.into_iter().map(Into::into).collect(),
            ignored_paths: value.ignored_paths.into_iter().map(Into::into).collect(),
            command_approvals: value
                .command_approvals
                .into_iter()
                .map(|approval| CommandApprovalDto {
                    key: CommandApprovalKeyDto {
                        jar_sha256: approval.identity.jar_sha256,
                        jvm_options: approval.identity.jvm_options,
                    },
                    jar: approval.jar,
                    approved_at: approval.approved_at.to_rfc3339(),
                })
                .collect(),
        }
    }
}

impl From<EditPluginSettingsDto> for EditPluginSettings {
    fn from(value: EditPluginSettingsDto) -> Self {
        Self {
            allowed_hosts: value.allowed_hosts,
            allowed_paths: value
                .allowed_paths
                .map(|paths| paths.into_iter().map(Into::into).collect()),
            ignored_paths: value
                .ignored_paths
                .map(|paths| paths.into_iter().map(Into::into).collect()),
            revoke_command_approvals: value
                .revoke_command_approvals
                .unwrap_or_default()
                .into_iter()
                .map(|key| CommandIdentity {
                    jar_sha256: key.jar_sha256,
                    jvm_options: key.jvm_options,
                })
                .collect(),
            revoke_local_hosts: value.revoke_local_hosts.unwrap_or_default(),
        }
    }
}

impl From<AllowedPath> for AllowedPathDto {
    fn from(value: AllowedPath) -> Self {
        Self {
            host_path: value.host,
            plugin_path: value.plugin.into(),
            read_only: value.read_only,
        }
    }
}

impl From<AllowedPathDto> for AllowedPathInput {
    fn from(value: AllowedPathDto) -> Self {
        Self {
            host: value.host_path,
            plugin: value.plugin_path,
            read_only: value.read_only,
        }
    }
}

impl From<IgnoredAllowedPath> for IgnoredAllowedPathDto {
    fn from(value: IgnoredAllowedPath) -> Self {
        Self {
            path: AllowedPathDto {
                host_path: value.path.host,
                plugin_path: value.path.plugin,
                read_only: value.path.read_only,
            },
            reason: value.reason.into(),
        }
    }
}

impl From<FolderAccess> for FolderAccessDto {
    fn from(value: FolderAccess) -> Self {
        match value {
            FolderAccess::AlreadyAllowed { folder } => Self::AlreadyAllowed { folder },
            FolderAccess::Needed { folder } => Self::Needed { folder },
            FolderAccess::Granted { folder } => Self::Granted { folder },
        }
    }
}

impl From<HostAccess> for HostAccessDto {
    fn from(value: HostAccess) -> Self {
        match value {
            HostAccess::AlreadyAllowed { host } => Self::AlreadyAllowed { host },
            HostAccess::Needed { host } => Self::Needed { host },
            HostAccess::Granted { host } => Self::Granted { host },
        }
    }
}

impl From<AllowedHostEntryRejection> for AllowedHostEntryRejectionDto {
    fn from(value: AllowedHostEntryRejection) -> Self {
        match value {
            AllowedHostEntryRejection::Empty => Self::Empty,
            AllowedHostEntryRejection::NotAHostName => Self::NotAHostName,
            AllowedHostEntryRejection::InvalidPattern => Self::InvalidPattern,
        }
    }
}

impl From<AllowedHostRejection> for AllowedHostRejectionDto {
    fn from(value: AllowedHostRejection) -> Self {
        match value {
            AllowedHostRejection::InvalidUrl => Self::InvalidUrl,
            AllowedHostRejection::UnsupportedScheme => Self::UnsupportedScheme,
            AllowedHostRejection::MissingHost => Self::MissingHost,
            AllowedHostRejection::NotPublic => Self::NotPublic,
            AllowedHostRejection::NotLocal => Self::NotLocal,
            AllowedHostRejection::Unresolved => Self::Unresolved,
            AllowedHostRejection::Wildcard => Self::Wildcard,
        }
    }
}

impl From<AllowedPathRejection> for AllowedPathRejectionDto {
    fn from(value: AllowedPathRejection) -> Self {
        match value {
            AllowedPathRejection::HostNotAbsolute => Self::HostNotAbsolute,
            AllowedPathRejection::HostNotAFolder => Self::HostNotAFolder,
            AllowedPathRejection::HostIsRoot => Self::HostIsRoot,
            AllowedPathRejection::HostIsHome => Self::HostIsHome,
            AllowedPathRejection::PluginNotAbsolute => Self::PluginNotAbsolute,
            AllowedPathRejection::PluginNotNormalized => Self::PluginNotNormalized,
            AllowedPathRejection::PluginIsRoot => Self::PluginIsRoot,
            AllowedPathRejection::PluginReserved => Self::PluginReserved,
            AllowedPathRejection::HostOverlapsCore => Self::HostOverlapsCore,
        }
    }
}

#[cfg(test)]
mod tests {
    use aether_core::features::plugins::{CommandApproval, CommandIdentity};
    use serde_json::json;

    use super::*;

    #[test]
    fn should_list_an_approval_with_its_key_beside_the_program_when_settings_are_sent() {
        let settings = PluginSettings {
            command_approvals: vec![CommandApproval {
                identity: CommandIdentity {
                    jar_sha256: "abc".into(),
                    jvm_options: vec!["-Xmx2G".into()],
                },
                plugin_sha256: "never-sent".into(),
                jar: "/cache/a.jar".into(),
                approved_at: "2026-11-16T10:00:00Z".parse().unwrap(),
            }],
            ..PluginSettings::default()
        };

        let value = serde_json::to_value(PluginSettingsDto::from(settings)).unwrap();

        assert_eq!(
            value["commandApprovals"],
            json!([{
                "jarSha256": "abc",
                "jvmOptions": ["-Xmx2G"],
                "jar": "/cache/a.jar",
                "approvedAt": "2026-11-16T10:00:00+00:00"
            }])
        );
    }

    #[test]
    fn should_revoke_nothing_when_the_edit_names_no_approvals() {
        let edit: EditPluginSettings = EditPluginSettingsDto::default().into();

        assert!(edit.revoke_command_approvals.is_empty());
    }
}
