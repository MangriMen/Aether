use crate::features::instance::app::dtos::capability_entry::CapabilityEntryDto;
use aether_core::shared::CapabilityEntry;
use serde::{Deserialize, Serialize};
use specta::Type;

use aether_core::features::instance::{
    CapabilityMetadata, ContentProviderCapabilityMetadata, ImporterCapabilityMetadata,
    UpdaterCapabilityMetadata,
};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityMetadataDto {
    /// Identifier for the capability (lowercase, kebab/underscore allowed).
    pub id: String,

    /// Display name of the capability.
    pub name: String,

    /// Optional detailed description of what this capability does.
    pub description: Option<String>,

    /// Optional icon file name or URL for the UI.
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImporterCapabilityMetadataDto {
    #[serde(flatten)]
    pub base: CapabilityMetadataDto,

    ///Optional field label shown in the importer UI.
    pub field_label: Option<String>,

    /// List of supported file extensions, e.g., [`zip`, `mrpack`].
    pub supported_extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterCapabilityMetadataDto {
    #[serde(flatten)]
    pub base: CapabilityMetadataDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentProviderCapabilityMetadataDto {
    #[serde(flatten)]
    pub base: CapabilityMetadataDto,

    /// Whether the provider supports installing individual items (e.g., a single mod or resource pack).
    pub supports_install_atomic: bool,

    /// Whether the provider supports installing complex modpacks or curated collections.
    pub supports_install_modpacks: bool,
}

impl From<CapabilityMetadata> for CapabilityMetadataDto {
    fn from(m: CapabilityMetadata) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            icon: m.icon,
        }
    }
}

impl From<ImporterCapabilityMetadata> for ImporterCapabilityMetadataDto {
    fn from(m: ImporterCapabilityMetadata) -> Self {
        Self {
            base: m.base.into(),
            field_label: m.field_label,
            supported_extensions: m.supported_extensions,
        }
    }
}

impl From<CapabilityEntry<ImporterCapabilityMetadata>>
    for CapabilityEntryDto<ImporterCapabilityMetadataDto>
{
    fn from(value: CapabilityEntry<ImporterCapabilityMetadata>) -> Self {
        CapabilityEntryDto {
            plugin_id: value.plugin_id,
            capability: value.capability.into(),
        }
    }
}

impl From<UpdaterCapabilityMetadata> for UpdaterCapabilityMetadataDto {
    fn from(m: UpdaterCapabilityMetadata) -> Self {
        Self {
            base: m.base.into(),
        }
    }
}

impl From<ContentProviderCapabilityMetadata> for ContentProviderCapabilityMetadataDto {
    fn from(m: ContentProviderCapabilityMetadata) -> Self {
        Self {
            base: m.base.into(),
            supports_install_atomic: m.supports_install_atomic,
            supports_install_modpacks: m.supports_install_modpacks,
        }
    }
}

impl From<CapabilityEntry<ContentProviderCapabilityMetadata>>
    for CapabilityEntryDto<ContentProviderCapabilityMetadataDto>
{
    fn from(value: CapabilityEntry<ContentProviderCapabilityMetadata>) -> Self {
        CapabilityEntryDto {
            plugin_id: value.plugin_id,
            capability: value.capability.into(),
        }
    }
}
