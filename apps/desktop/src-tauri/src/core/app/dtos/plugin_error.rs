use aether_core::features::plugins::PluginError;
use serde::Serialize;
use specta::Type;

use crate::features::plugins::{LoadConfigDto, LoadConfigTypeDto};

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
    LoadFailed {
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
    LoaderNotFound {
        config_type: LoadConfigTypeDto,
    },
    InvalidConfig {
        config: LoadConfigDto,
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
    HashComputationFailed,
    Settings {
        details: String,
    },
    Storage {
        details: String,
    },
}

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
            PluginError::LoadFailed { plugin_id, reason } => Self::LoadFailed {
                plugin_id: plugin_id.clone(),
                reason: reason.clone(),
            },
            PluginError::FunctionCallFailed {
                function_name,
                plugin_id,
                error,
            } => Self::FunctionCallFailed {
                function_name: function_name.clone(),
                plugin_id: plugin_id.clone(),
                error: error.clone(),
            },
            PluginError::ManifestNotFound { path } => Self::ManifestNotFound { path: path.clone() },
            PluginError::InvalidManifestFormat { error } => Self::InvalidManifestFormat {
                error: error.clone(),
            },
            PluginError::LoaderNotFound { config_type } => Self::LoaderNotFound {
                config_type: (*config_type).into(),
            },
            PluginError::InvalidConfig { config } => Self::InvalidConfig {
                config: config.clone().into(),
            },
            PluginError::ExtractionFailed { from } => Self::ExtractionFailed { from: from.clone() },
            PluginError::InvalidExtractionFormat => Self::InvalidExtractionFormat,
            PluginError::FileExtractionFailed { from } => {
                Self::FileExtractionFailed { from: from.clone() }
            }
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
            PluginError::HashComputationFailed => Self::HashComputationFailed,
            PluginError::Settings(err) => Self::Settings {
                details: err.to_string(),
            },
            PluginError::Storage(err) => Self::Storage {
                details: err.to_string(),
            },
        }
    }
}
