use aether_core::features::plugins::{PluginError, PluginProviderError};
use serde::Serialize;
use specta::Type;

use crate::features::plugins::{
    AllowedHostRejectionDto, AllowedPathRejectionDto, LoadConfigTypeDto,
};

#[derive(Debug, Serialize, Type)]
#[serde(tag = "code", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PluginErrorDto {
    NotFound {
        plugin_id: String,
    },
    AlreadyLoaded {
        plugin_id: String,
    },
    LoadingInProgress {
        plugin_id: String,
    },
    UnloadingInProgress {
        plugin_id: String,
    },
    AlreadyUnloaded {
        plugin_id: String,
    },
    Busy {
        plugin_id: String,
    },
    Stopped {
        plugin_id: String,
    },
    IdMismatch {
        plugin_id: String,
        archive_id: String,
    },
    LoadFailed {
        plugin_id: String,
        reason: String,
    },
    IncompatibleApiVersion {
        plugin_id: String,
        reason: String,
    },
    FunctionCallFailed {
        function_name: String,
        plugin_id: String,
        error: String,
    },
    ManifestNotFound {
        path: String,
    },
    InvalidManifestFormat {
        error: String,
    },
    Manifest {
        message: String,
    },
    LoaderNotFound {
        config_type: LoadConfigTypeDto,
    },
    ExtractionFailed {
        from: String,
    },
    InvalidExtractionFormat,
    FileExtractionFailed {
        from: String,
    },
    CapabilityRegistrationFailed {
        capability_type: String,
        capability_id: String,
    },
    CapabilityCancelRegistrationFailed {
        capability_type: String,
        capability_id: String,
    },
    AccessViolation {
        plugin_id: String,
        path: String,
    },
    InvalidAllowedPath {
        plugin_id: String,
        path: String,
        reason: AllowedPathRejectionDto,
    },
    InvalidAllowedHost {
        plugin_id: String,
        url: String,
        reason: AllowedHostRejectionDto,
    },
    HashComputationFailed,
    Settings {
        details: String,
    },
    Storage {
        details: String,
    },
    // ── Provider errors (provider-agnostic) ──
    ProviderFetchError {
        source_type: String,
        details: String,
    },
    ProviderNoAssets {
        source_type: String,
    },
    NoRemoteSource {
        plugin_id: String,
    },
    DownloadFailed {
        url: String,
        details: String,
    },
    // ── Rate limiting ──
    ProviderRateLimited {
        source_type: String,
        retry_after: Option<u32>,
    },
}

#[allow(clippy::too_many_lines)]
impl From<&PluginError> for PluginErrorDto {
    fn from(value: &PluginError) -> Self {
        match value {
            PluginError::NotFound { plugin_id } => Self::NotFound {
                plugin_id: plugin_id.clone(),
            },
            PluginError::AlreadyLoaded { plugin_id } => Self::AlreadyLoaded {
                plugin_id: plugin_id.clone(),
            },
            PluginError::LoadingInProgress { plugin_id } => Self::LoadingInProgress {
                plugin_id: plugin_id.clone(),
            },
            PluginError::UnloadingInProgress { plugin_id } => Self::UnloadingInProgress {
                plugin_id: plugin_id.clone(),
            },
            PluginError::AlreadyUnloaded { plugin_id } => Self::AlreadyUnloaded {
                plugin_id: plugin_id.clone(),
            },
            PluginError::Busy { plugin_id } => Self::Busy {
                plugin_id: plugin_id.clone(),
            },
            PluginError::Stopped { plugin_id } => Self::Stopped {
                plugin_id: plugin_id.clone(),
            },
            PluginError::IdMismatch {
                plugin_id,
                archive_id,
            } => Self::IdMismatch {
                plugin_id: plugin_id.clone(),
                archive_id: archive_id.clone(),
            },
            PluginError::LoadFailed { plugin_id, reason } => Self::LoadFailed {
                plugin_id: plugin_id.clone(),
                reason: reason.clone(),
            },
            PluginError::IncompatibleApiVersion { plugin_id, reason } => {
                Self::IncompatibleApiVersion {
                    plugin_id: plugin_id.clone(),
                    reason: reason.clone(),
                }
            }
            PluginError::FunctionCallFailed {
                function_name,
                plugin_id,
                error,
            } => Self::FunctionCallFailed {
                function_name: function_name.clone(),
                plugin_id: plugin_id.clone(),
                error: error.to_string(),
            },
            PluginError::Manifest(err) => Self::Manifest {
                message: err.to_string(),
            },
            PluginError::LoaderNotFound { config_type } => Self::LoaderNotFound {
                config_type: (*config_type).into(),
            },
            PluginError::CapabilityRegistrationFailed {
                capability_type,
                capability_id,
            } => Self::CapabilityRegistrationFailed {
                capability_type: capability_type.to_string(),
                capability_id: capability_id.clone(),
            },
            PluginError::CapabilityCancelRegistrationFailed {
                capability_type,
                capability_id,
            } => Self::CapabilityCancelRegistrationFailed {
                capability_type: capability_type.to_string(),
                capability_id: capability_id.clone(),
            },
            PluginError::AccessViolation { plugin_id, path } => Self::AccessViolation {
                plugin_id: plugin_id.clone(),
                path: path.clone(),
            },
            PluginError::InvalidAllowedPath {
                plugin_id,
                path,
                reason,
            } => Self::InvalidAllowedPath {
                plugin_id: plugin_id.clone(),
                path: path.clone(),
                reason: (*reason).into(),
            },
            PluginError::InvalidAllowedHost {
                plugin_id,
                url,
                reason,
            } => Self::InvalidAllowedHost {
                plugin_id: plugin_id.clone(),
                url: url.clone(),
                reason: (*reason).into(),
            },
            PluginError::HashComputationFailed => Self::HashComputationFailed,
            PluginError::Settings(err) => Self::Settings {
                details: err.to_string(),
            },
            PluginError::Storage(err) => Self::Storage {
                details: err.to_string(),
            },
            PluginError::Provider(err) => err.into(),
        }
    }
}

impl From<&PluginProviderError> for PluginErrorDto {
    fn from(value: &PluginProviderError) -> Self {
        match value {
            PluginProviderError::ManifestNotFound { path } => {
                Self::ManifestNotFound { path: path.clone() }
            }
            PluginProviderError::InvalidManifestFormat { error } => Self::InvalidManifestFormat {
                error: error.clone(),
            },
            PluginProviderError::ExtractionFailed { from } => {
                Self::ExtractionFailed { from: from.clone() }
            }
            PluginProviderError::InvalidExtractionFormat => Self::InvalidExtractionFormat,
            PluginProviderError::FileExtractionFailed { from } => {
                Self::FileExtractionFailed { from: from.clone() }
            }
            PluginProviderError::ProviderFetchError {
                source_type,
                details,
            } => Self::ProviderFetchError {
                source_type: source_type.to_string(),
                details: details.clone(),
            },
            PluginProviderError::ProviderNoAssets { source_type } => Self::ProviderNoAssets {
                source_type: source_type.to_string(),
            },
            PluginProviderError::NoRemoteSource { plugin_id } => Self::NoRemoteSource {
                plugin_id: plugin_id.clone(),
            },
            PluginProviderError::DownloadFailed { url, details } => Self::DownloadFailed {
                url: url.clone(),
                details: details.clone(),
            },
            PluginProviderError::ProviderRateLimited {
                source_type,
                retry_after,
            } => Self::ProviderRateLimited {
                source_type: source_type.to_string(),
                retry_after: *retry_after,
            },
            PluginProviderError::Manifest(err) => Self::Manifest {
                message: err.to_string(),
            },
            PluginProviderError::Storage(err) => Self::Storage {
                details: err.to_string(),
            },
        }
    }
}
