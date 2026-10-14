use std::path::PathBuf;

use aether_core::features::plugins::{
    AllowedHostRejection, AllowedPath, AllowedPathRejection, EditPluginSettings, FolderAccess,
    HostAccess, PluginSettings,
};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Default, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginSettingsDto {
    #[serde(default)]
    pub allowed_hosts: Vec<String>,

    #[serde(default)]
    pub allowed_paths: Vec<AllowedPathDto>,
}

#[derive(Serialize, Deserialize, Debug, Default, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditPluginSettingsDto {
    #[specta(optional)]
    pub allowed_hosts: Option<Vec<String>>,
    #[specta(optional)]
    pub allowed_paths: Option<Vec<AllowedPathDto>>,
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

#[derive(Serialize, Debug, Type)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum FolderAccessDto {
    /// The plugin could already read the file.
    AlreadyAllowed { folder: PathBuf },
    /// A dry run: the plugin cannot read the file yet; nothing was changed.
    Needed { folder: PathBuf },
    /// The folder was added read-only and the plugin restarted with it.
    Granted { folder: PathBuf },
}

#[derive(Serialize, Debug, Type)]
#[serde(rename_all = "camelCase", tag = "type")]
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
pub enum AllowedHostRejectionDto {
    InvalidUrl,
    UnsupportedScheme,
    MissingHost,
    NotPublic,
    Wildcard,
}

#[derive(Serialize, Debug, Clone, Copy, Type)]
#[serde(rename_all = "snake_case")]
pub enum AllowedPathRejectionDto {
    HostNotAbsolute,
    HostNotAFolder,
    HostIsRoot,
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
            allowed_paths: value.allowed_paths.into_iter().map(Into::into).collect(),
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
        }
    }
}

impl From<AllowedPath> for AllowedPathDto {
    fn from(value: AllowedPath) -> Self {
        Self {
            host_path: value.host,
            plugin_path: value.plugin,
            read_only: value.read_only,
        }
    }
}

impl From<AllowedPathDto> for AllowedPath {
    fn from(value: AllowedPathDto) -> Self {
        Self {
            host: value.host_path,
            plugin: value.plugin_path,
            read_only: value.read_only,
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

impl From<AllowedHostRejection> for AllowedHostRejectionDto {
    fn from(value: AllowedHostRejection) -> Self {
        match value {
            AllowedHostRejection::InvalidUrl => Self::InvalidUrl,
            AllowedHostRejection::UnsupportedScheme => Self::UnsupportedScheme,
            AllowedHostRejection::MissingHost => Self::MissingHost,
            AllowedHostRejection::NotPublic => Self::NotPublic,
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
            AllowedPathRejection::PluginNotAbsolute => Self::PluginNotAbsolute,
            AllowedPathRejection::PluginNotNormalized => Self::PluginNotNormalized,
            AllowedPathRejection::PluginIsRoot => Self::PluginIsRoot,
            AllowedPathRejection::PluginReserved => Self::PluginReserved,
            AllowedPathRejection::HostOverlapsCore => Self::HostOverlapsCore,
        }
    }
}
