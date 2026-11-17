use aether_core::features::plugins::{
    AllowedHostChange, CapabilityChange, CapabilityKind, HostFunctionChange, PluginUpdateDiff,
    PluginUpdatePreview,
};
use serde::Serialize;
use specta::Type;

use super::{HostFunctionRiskDto, ProviderPluginPreviewDto};

/// A release of an installed plugin as shown before the update to it. The update is given
/// `release.archive_sha256` back, so it installs the archive shown here or nothing.
#[derive(Debug, Clone, Serialize, Type)]
pub struct PluginUpdatePreviewDto {
    pub release: ProviderPluginPreviewDto,
    /// The version installed now, which `diff` is taken against.
    pub installed_version: String,
    pub diff: PluginUpdateDiffDto,
}

impl From<PluginUpdatePreview> for PluginUpdatePreviewDto {
    fn from(value: PluginUpdatePreview) -> Self {
        Self {
            release: value.release.into(),
            installed_version: value.installed_version,
            diff: value.diff.into(),
        }
    }
}

/// What the release asks for that the installed version does not ("added"), and what it gives up
/// ("removed"). Computed and classified by the core: the frontend only shows it.
#[derive(Debug, Clone, Serialize, Type)]
pub struct PluginUpdateDiffDto {
    pub added_host_functions: Vec<HostFunctionChangeDto>,
    pub removed_host_functions: Vec<HostFunctionChangeDto>,
    /// Risks the release carries that the installed version does not.
    pub added_risks: Vec<HostFunctionRiskDto>,
    pub added_allowed_hosts: Vec<AllowedHostChangeDto>,
    pub removed_allowed_hosts: Vec<String>,
    pub added_capabilities: Vec<CapabilityChangeDto>,
    pub removed_capabilities: Vec<CapabilityChangeDto>,
    /// Capabilities both have under the same kind and id but declare differently, such as a
    /// content provider that now serves another platform.
    pub changed_capabilities: Vec<CapabilityChangeDto>,
    /// The release may do something the user is warned about that the installed version may not.
    pub adds_risk: bool,
    /// Both versions ask for the same.
    pub is_empty: bool,
}

impl From<PluginUpdateDiff> for PluginUpdateDiffDto {
    fn from(value: PluginUpdateDiff) -> Self {
        let adds_risk = value.adds_risk();
        let is_empty = value.is_empty();
        Self {
            added_host_functions: value
                .added_host_functions
                .into_iter()
                .map(Into::into)
                .collect(),
            removed_host_functions: value
                .removed_host_functions
                .into_iter()
                .map(Into::into)
                .collect(),
            added_risks: value.added_risks.into_iter().map(Into::into).collect(),
            added_allowed_hosts: value
                .added_allowed_hosts
                .into_iter()
                .map(Into::into)
                .collect(),
            removed_allowed_hosts: value.removed_allowed_hosts,
            added_capabilities: value
                .added_capabilities
                .into_iter()
                .map(Into::into)
                .collect(),
            removed_capabilities: value
                .removed_capabilities
                .into_iter()
                .map(Into::into)
                .collect(),
            changed_capabilities: value
                .changed_capabilities
                .into_iter()
                .map(Into::into)
                .collect(),
            adds_risk,
            is_empty,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct HostFunctionChangeDto {
    pub name: String,
    pub risk: Option<HostFunctionRiskDto>,
}

impl From<HostFunctionChange> for HostFunctionChangeDto {
    fn from(value: HostFunctionChange) -> Self {
        Self {
            name: value.name,
            risk: value.risk.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct AllowedHostChangeDto {
    pub entry: String,
    /// The entry admits far more than one site.
    pub wide: bool,
}

impl From<AllowedHostChange> for AllowedHostChangeDto {
    fn from(value: AllowedHostChange) -> Self {
        Self {
            entry: value.entry,
            wide: value.wide,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityKindDto {
    Importer,
    Updater,
    ContentProvider,
}

impl From<CapabilityKind> for CapabilityKindDto {
    fn from(value: CapabilityKind) -> Self {
        match value {
            CapabilityKind::Importer => Self::Importer,
            CapabilityKind::Updater => Self::Updater,
            CapabilityKind::ContentProvider => Self::ContentProvider,
        }
    }
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct CapabilityChangeDto {
    pub kind: CapabilityKindDto,
    pub id: String,
    pub name: String,
}

impl From<CapabilityChange> for CapabilityChangeDto {
    fn from(value: CapabilityChange) -> Self {
        Self {
            kind: value.kind.into(),
            id: value.id,
            name: value.name,
        }
    }
}
