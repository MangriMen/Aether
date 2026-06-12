use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::features::plugins::{
    PathMapping, PluginCapabilities, PluginContentProviderCapability, PluginImporterCapability,
    PluginSettings, PluginUpdaterCapability, ProviderHandlers,
};

/// DTO for `PluginCapabilities` that can be read/written as JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilitiesV1 {
    pub importers: Option<Vec<PluginImporterCapabilityV1>>,
    pub updaters: Option<Vec<PluginUpdaterCapabilityV1>>,
    pub content_providers: Option<Vec<PluginContentProviderCapabilityV1>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginImporterCapabilityV1 {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub field_label: Option<String>,
    pub supported_extensions: Vec<String>,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdaterCapabilityV1 {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContentProviderCapabilityV1 {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub supports_install_atomic: bool,
    pub supports_install_modpacks: bool,
    pub handlers: ProviderHandlersV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHandlersV1 {
    pub search: String,
    pub get_content: String,
    pub list_version: Option<String>,
    pub install_atomic: String,
    pub install_modpack: Option<String>,
    pub check_compatibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettingsV1 {
    pub allowed_hosts: Vec<String>,
    pub allowed_paths: Vec<(String, String)>,
}

// ── Converters V1 → Domain ──

impl From<PluginCapabilitiesV1> for PluginCapabilities {
    fn from(v1: PluginCapabilitiesV1) -> Self {
        Self {
            importers: v1
                .importers
                .unwrap_or_default()
                .into_iter()
                .map(|i| PluginImporterCapability {
                    metadata: crate::features::instance::ImporterCapabilityMetadata {
                        base: crate::features::instance::CapabilityMetadata {
                            id: i.id,
                            name: i.name,
                            description: i.description,
                            icon: i.icon,
                        },
                        field_label: i.field_label,
                        supported_extensions: i.supported_extensions,
                    },
                    handler: i.handler,
                })
                .collect(),
            updaters: v1
                .updaters
                .unwrap_or_default()
                .into_iter()
                .map(|u| PluginUpdaterCapability {
                    metadata: crate::features::instance::UpdaterCapabilityMetadata {
                        base: crate::features::instance::CapabilityMetadata {
                            id: u.id,
                            name: u.name,
                            description: u.description,
                            icon: u.icon,
                        },
                    },
                    handler: u.handler,
                })
                .collect(),
            content_providers: v1
                .content_providers
                .unwrap_or_default()
                .into_iter()
                .map(|cp| PluginContentProviderCapability {
                    metadata: crate::features::instance::ContentProviderCapabilityMetadata {
                        base: crate::features::instance::CapabilityMetadata {
                            id: cp.id,
                            name: cp.name,
                            description: cp.description,
                            icon: cp.icon,
                        },
                        supports_install_atomic: cp.supports_install_atomic,
                        supports_install_modpacks: cp.supports_install_modpacks,
                    },
                    handlers: ProviderHandlers {
                        search: cp.handlers.search,
                        get_content: cp.handlers.get_content,
                        list_version: cp.handlers.list_version,
                        install_atomic: cp.handlers.install_atomic,
                        install_modpack: cp.handlers.install_modpack,
                        check_compatibility: cp.handlers.check_compatibility,
                    },
                })
                .collect(),
        }
    }
}

impl From<PluginSettingsV1> for PluginSettings {
    fn from(v1: PluginSettingsV1) -> Self {
        Self {
            allowed_hosts: v1.allowed_hosts,
            allowed_paths: v1
                .allowed_paths
                .into_iter()
                .map(|(host, virt)| PathMapping(host, PathBuf::from(virt)))
                .collect(),
        }
    }
}

// ── Converters Domain → V1 ──

impl From<PluginCapabilities> for PluginCapabilitiesV1 {
    fn from(v: PluginCapabilities) -> Self {
        Self {
            importers: Some(
                v.importers
                    .into_iter()
                    .map(|i| PluginImporterCapabilityV1 {
                        id: i.metadata.base.id.clone(),
                        name: i.metadata.base.name.clone(),
                        description: i.metadata.base.description.clone(),
                        icon: i.metadata.base.icon.clone(),
                        field_label: i.metadata.field_label.clone(),
                        supported_extensions: i.metadata.supported_extensions.clone(),
                        handler: i.handler,
                    })
                    .collect(),
            ),
            updaters: Some(
                v.updaters
                    .into_iter()
                    .map(|u| PluginUpdaterCapabilityV1 {
                        id: u.metadata.base.id.clone(),
                        name: u.metadata.base.name.clone(),
                        description: u.metadata.base.description.clone(),
                        icon: u.metadata.base.icon.clone(),
                        handler: u.handler,
                    })
                    .collect(),
            ),
            content_providers: Some(
                v.content_providers
                    .into_iter()
                    .map(|cp| PluginContentProviderCapabilityV1 {
                        id: cp.metadata.base.id.clone(),
                        name: cp.metadata.base.name.clone(),
                        description: cp.metadata.base.description.clone(),
                        icon: cp.metadata.base.icon.clone(),
                        supports_install_atomic: cp.metadata.supports_install_atomic,
                        supports_install_modpacks: cp.metadata.supports_install_modpacks,
                        handlers: ProviderHandlersV1 {
                            search: cp.handlers.search,
                            get_content: cp.handlers.get_content,
                            list_version: cp.handlers.list_version,
                            install_atomic: cp.handlers.install_atomic,
                            install_modpack: cp.handlers.install_modpack,
                            check_compatibility: cp.handlers.check_compatibility,
                        },
                    })
                    .collect(),
            ),
        }
    }
}

impl From<PluginSettings> for PluginSettingsV1 {
    fn from(v: PluginSettings) -> Self {
        Self {
            allowed_hosts: v.allowed_hosts,
            allowed_paths: v
                .allowed_paths
                .into_iter()
                .map(|pm| (pm.0, pm.1.to_string_lossy().to_string()))
                .collect(),
        }
    }
}
