use aether_core::shared::capability::CapabilityEntry;
use serde::{Deserialize, Serialize};
use specta::Type;

use aether_core::features::instance::{
    CapabilityMetadata, ContentProviderCapabilityMetadata, PackManagerCapabilityMetadata,
};

use super::CapabilityEntryDto;

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
pub struct ContentProviderCapabilityMetadataDto {
    #[serde(flatten)]
    pub base: CapabilityMetadataDto,

    /// Whether the provider supports installing individual items (e.g., a single mod or resource pack).
    pub supports_install_atomic: bool,

    /// Whether the provider supports installing complex modpacks or curated collections.
    pub supports_install_modpacks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PackManagerCapabilityMetadataDto {
    #[serde(flatten)]
    pub base: CapabilityMetadataDto,

    /// Whether the manager supports installing packs.
    pub supports_install: bool,

    /// Whether the manager supports updating existing packs.
    pub supports_update: bool,

    /// Whether the manager supports checking for updates.
    pub supports_check_updates: bool,

    /// Optional label for a file/URL input field shown in the "Import" UI.
    pub field_label: Option<String>,

    /// List of supported file extensions for import, e.g. [`"toml"`, `"mrpack"`].
    pub supported_extensions: Vec<String>,
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

impl From<aether_core::features::plugins::CapabilityMetadataDto> for CapabilityMetadataDto {
    fn from(dto: aether_core::features::plugins::CapabilityMetadataDto) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            description: dto.description,
            icon: dto.icon,
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

impl From<aether_core::features::plugins::ContentProviderCapabilityMetadataDto>
    for ContentProviderCapabilityMetadataDto
{
    fn from(dto: aether_core::features::plugins::ContentProviderCapabilityMetadataDto) -> Self {
        Self {
            base: dto.base.into(),
            supports_install_atomic: dto.supports_install_atomic,
            supports_install_modpacks: dto.supports_install_modpacks,
        }
    }
}

impl From<PackManagerCapabilityMetadata> for PackManagerCapabilityMetadataDto {
    fn from(m: PackManagerCapabilityMetadata) -> Self {
        Self {
            base: m.base.into(),
            supports_install: m.supports_install,
            supports_update: m.supports_update,
            supports_check_updates: m.supports_check_updates,
            field_label: m.field_label,
            supported_extensions: m.supported_extensions,
        }
    }
}

impl From<aether_core::features::plugins::PackManagerCapabilityMetadataDto>
    for PackManagerCapabilityMetadataDto
{
    fn from(dto: aether_core::features::plugins::PackManagerCapabilityMetadataDto) -> Self {
        Self {
            base: dto.base.into(),
            supports_install: dto.supports_install,
            supports_update: dto.supports_update,
            supports_check_updates: dto.supports_check_updates,
            field_label: dto.field_label,
            supported_extensions: dto.supported_extensions,
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

impl From<CapabilityEntry<PackManagerCapabilityMetadata>>
    for CapabilityEntryDto<PackManagerCapabilityMetadataDto>
{
    fn from(value: CapabilityEntry<PackManagerCapabilityMetadata>) -> Self {
        CapabilityEntryDto {
            plugin_id: value.plugin_id,
            capability: value.capability.into(),
        }
    }
}
