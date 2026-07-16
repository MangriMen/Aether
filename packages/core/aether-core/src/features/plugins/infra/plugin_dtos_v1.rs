use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::features::plugins::{
    PackManagerHandlers, PathMapping, PluginCapabilities, PluginContentProviderCapability,
    PluginPackManagerCapability, PluginSettings, ProviderHandlers,
};

/// DTO for `PluginCapabilities` that can be read/written as JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCapabilitiesV1 {
    pub content_providers: Option<Vec<PluginContentProviderCapabilityV1>>,
    #[serde(default)]
    pub pack_managers: Option<Vec<PluginPackManagerCapabilityV1>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct ProviderHandlersV1 {
    pub search: String,
    pub get_content: String,
    pub list_version: Option<String>,
    pub install_atomic: String,
    pub install_modpack: Option<String>,
    pub check_compatibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginPackManagerCapabilityV1 {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    #[serde(default = "return_true")]
    pub supports_install: bool,
    #[serde(default)]
    pub supports_update: bool,
    #[serde(default)]
    pub supports_check_updates: bool,
    pub field_label: Option<String>,
    #[serde(default)]
    pub supported_extensions: Vec<String>,
    pub handlers: PackManagerHandlersV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackManagerHandlersV1 {
    pub install: String,
    pub update: Option<String>,
    pub check_updates: Option<String>,
    #[serde(default)]
    pub resolve_metadata: Option<String>,
}

fn return_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSettingsV1 {
    pub allowed_hosts: Vec<String>,
    pub allowed_paths: Vec<(String, String)>,

    #[serde(default)]
    pub force_enabled_at_api_version: Option<String>,
}

// ── Converters V1 → Domain ──

impl From<PluginCapabilitiesV1> for PluginCapabilities {
    fn from(v1: PluginCapabilitiesV1) -> Self {
        Self {
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
            pack_managers: v1
                .pack_managers
                .unwrap_or_default()
                .into_iter()
                .map(|pm| PluginPackManagerCapability {
                    metadata: crate::features::instance::PackManagerCapabilityMetadata {
                        base: crate::features::instance::CapabilityMetadata {
                            id: pm.id,
                            name: pm.name,
                            description: pm.description,
                            icon: pm.icon,
                        },
                        supports_install: pm.supports_install,
                        supports_update: pm.supports_update,
                        supports_check_updates: pm.supports_check_updates,
                        field_label: pm.field_label,
                        supported_extensions: pm.supported_extensions,
                    },
                    handlers: PackManagerHandlers {
                        install: pm.handlers.install,
                        update: pm.handlers.update,
                        check_updates: pm.handlers.check_updates,
                        resolve_metadata: pm.handlers.resolve_metadata,
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
            force_enabled_at_api_version: v1.force_enabled_at_api_version,
        }
    }
}

// ── Converters Domain → V1 ──

impl From<PluginCapabilities> for PluginCapabilitiesV1 {
    fn from(v: PluginCapabilities) -> Self {
        Self {
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
            pack_managers: Some(
                v.pack_managers
                    .into_iter()
                    .map(|pm| PluginPackManagerCapabilityV1 {
                        id: pm.metadata.base.id.clone(),
                        name: pm.metadata.base.name.clone(),
                        description: pm.metadata.base.description.clone(),
                        icon: pm.metadata.base.icon.clone(),
                        supports_install: pm.metadata.supports_install,
                        supports_update: pm.metadata.supports_update,
                        supports_check_updates: pm.metadata.supports_check_updates,
                        field_label: pm.metadata.field_label.clone(),
                        supported_extensions: pm.metadata.supported_extensions.clone(),
                        handlers: PackManagerHandlersV1 {
                            install: pm.handlers.install,
                            update: pm.handlers.update,
                            check_updates: pm.handlers.check_updates,
                            resolve_metadata: pm.handlers.resolve_metadata,
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
            force_enabled_at_api_version: v.force_enabled_at_api_version,
        }
    }
}
