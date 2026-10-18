use serde::{Deserialize, Serialize};

use crate::features::plugins::{
    AllowedPath, PluginCapabilities, PluginContentProviderCapability, PluginImporterCapability,
    PluginSettings, PluginUpdaterCapability, ProviderHandlers,
};

/// DTO for `PluginCapabilities` that can be read/written as JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCapabilitiesV1 {
    pub importers: Option<Vec<PluginImporterCapabilityV1>>,
    pub updaters: Option<Vec<PluginUpdaterCapabilityV1>>,
    pub content_providers: Option<Vec<PluginContentProviderCapabilityV1>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginImporterCapabilityV1 {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub field_label: Option<String>,
    #[serde(default)]
    pub supported_extensions: Vec<String>,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginUpdaterCapabilityV1 {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub handler: String,
}

/// A content provider entry of `capabilities.json`.
///
/// What the provider supports is read from `handlers` alone; keys the core does not know are
/// ignored.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginContentProviderCapabilityV1 {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    pub handlers: ProviderHandlersV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderHandlersV1 {
    pub search: String,
    pub get_content: String,
    pub list_version: Option<String>,
    pub check_compatibility: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolve_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolve_install_plan: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolve_pack_metadata: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub get_contents_metadata: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub get_versions_metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSettingsV1 {
    pub allowed_hosts: Vec<String>,
    pub allowed_paths: Vec<AllowedPathV1>,

    #[serde(default)]
    pub force_enabled_at_api_version: Option<String>,
}

/// One folder of `PluginSettingsV1::allowed_paths`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AllowedPathV1 {
    #[serde(rename_all = "camelCase")]
    Mapping {
        host: String,
        plugin: String,
        #[serde(default = "read_only_by_default")]
        read_only: bool,
    },
    /// `[host, plugin]`, the format without rights. Read as read-only: write access has to be
    /// given explicitly.
    Legacy(String, String),
}

fn read_only_by_default() -> bool {
    true
}

impl From<AllowedPathV1> for AllowedPath {
    fn from(v1: AllowedPathV1) -> Self {
        match v1 {
            AllowedPathV1::Mapping {
                host,
                plugin,
                read_only,
            } => Self {
                host,
                plugin,
                read_only,
            },
            AllowedPathV1::Legacy(host, plugin) => Self {
                host,
                plugin,
                read_only: true,
            },
        }
    }
}

impl From<AllowedPath> for AllowedPathV1 {
    fn from(path: AllowedPath) -> Self {
        Self::Mapping {
            host: path.host,
            plugin: path.plugin,
            read_only: path.read_only,
        }
    }
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
                        platform: cp.platform,
                    },
                    handlers: ProviderHandlers {
                        search: cp.handlers.search,
                        get_content: cp.handlers.get_content,
                        list_version: cp.handlers.list_version,
                        check_compatibility: cp.handlers.check_compatibility,
                        resolve_file: cp.handlers.resolve_file,
                        resolve_install_plan: cp.handlers.resolve_install_plan,
                        resolve_pack_metadata: cp.handlers.resolve_pack_metadata,
                        get_contents_metadata: cp.handlers.get_contents_metadata,
                        get_versions_metadata: cp.handlers.get_versions_metadata,
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
            allowed_paths: v1.allowed_paths.into_iter().map(Into::into).collect(),
            force_enabled_at_api_version: v1.force_enabled_at_api_version,
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
                        platform: cp.metadata.platform.clone(),
                        handlers: ProviderHandlersV1 {
                            search: cp.handlers.search,
                            get_content: cp.handlers.get_content,
                            list_version: cp.handlers.list_version,
                            check_compatibility: cp.handlers.check_compatibility,
                            resolve_file: cp.handlers.resolve_file,
                            resolve_install_plan: cp.handlers.resolve_install_plan,
                            resolve_pack_metadata: cp.handlers.resolve_pack_metadata,
                            get_contents_metadata: cp.handlers.get_contents_metadata,
                            get_versions_metadata: cp.handlers.get_versions_metadata,
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
            allowed_paths: v.allowed_paths.into_iter().map(Into::into).collect(),
            force_enabled_at_api_version: v.force_enabled_at_api_version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_read_a_folder_as_read_only_when_it_was_saved_before_rights_existed() {
        let v1: PluginSettingsV1 = toml::from_str(
            r#"
            allowedHosts = []
            allowedPaths = [["D:\\", "/mnt/d"]]
            "#,
        )
        .unwrap();

        assert_eq!(
            PluginSettings::from(v1).allowed_paths,
            [AllowedPath {
                host: r"D:\".to_owned(),
                plugin: "/mnt/d".to_owned(),
                read_only: true,
            }]
        );
    }

    #[test]
    fn should_keep_the_rights_of_each_folder_when_settings_are_saved() {
        let settings = PluginSettings {
            allowed_paths: vec![
                AllowedPath {
                    host: r"D:\packs".to_owned(),
                    plugin: "/mnt/d/packs".to_owned(),
                    read_only: true,
                },
                AllowedPath {
                    host: r"D:\out".to_owned(),
                    plugin: "/mnt/d/out".to_owned(),
                    read_only: false,
                },
            ],
            force_enabled_at_api_version: Some("0.2.0".to_owned()),
            ..PluginSettings::default()
        };

        let written = toml::to_string(&PluginSettingsV1::from(settings.clone())).unwrap();
        let read: PluginSettingsV1 = toml::from_str(&written).unwrap();

        assert_eq!(
            PluginSettings::from(read).allowed_paths,
            settings.allowed_paths
        );
    }
}
