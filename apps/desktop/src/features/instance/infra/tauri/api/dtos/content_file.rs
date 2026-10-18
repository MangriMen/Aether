use aether_core::features::instance::{
    ContentFile, ContentFileUpdateInfo, ContentMetadata, ContentVersionMetadata,
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use specta::Type;

use crate::{
    core::format_asset_url,
    features::instance::infra::{ContentTypeDto, ProviderIdDto},
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentFileDto {
    pub content_path: String,
    pub content_type: ContentTypeDto,
    pub disabled: bool,
    pub file_name: String,
    pub hash: String,
    pub name: Option<String>,
    #[specta(type = String)]
    #[serde_as(as = "DisplayFromStr")]
    pub size: u64,
    pub version: Option<String>,
    pub update_provider_id: Option<ProviderIdDto>,
    /// The update each provider offers for the file, ordered by provider.
    pub update: Option<Vec<ContentFileUpdateDto>>,
    /// The file's project as its provider describes it, when cached. Filled in later when
    /// missing: the instance's `contentSynced` event tells when to list again.
    pub metadata: Option<ContentMetadataDto>,
    /// The version the file was installed as, as its provider describes it, when cached: what
    /// to show in place of `version`, the provider's id of it. Filled in later as `metadata` is.
    pub version_metadata: Option<ContentVersionMetadataDto>,
    /// Whether `versionMetadata` is being fetched now: the instance's `contentSynced` event tells
    /// when the fetch is over. `false` when nothing is to come.
    pub version_metadata_pending: bool,
    /// Whether `metadata` lacks its author and is being fetched now: the instance's
    /// `contentSynced` event tells when the fetch is over. `false` when nothing is to come, or
    /// the author is known.
    pub metadata_pending: bool,
}

/// What a provider says about the project of an installed file.
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentMetadataDto {
    pub content_id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub icon_url: Option<String>,
    /// The icon as cached on disk, as a URL of the app's asset protocol, as the instance's
    /// `iconPath`. Shown in place of `iconUrl` when set; `None` until the icon has arrived.
    pub icon_path: Option<String>,
}

/// What a provider says about the version of an installed file.
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersionMetadataDto {
    /// The provider's id of the version, as `ContentFileDto::version` holds it.
    pub version_id: String,
    /// The version as its authors number it (`0.6.13+mc1.21.4`).
    pub version_number: String,
    /// The version's title (`Sodium 0.6.13`), if it has one.
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentFileUpdateInfoDto {
    pub content_id: String,
    pub version: String,
}

/// The update one provider offers for an installed file.
#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentFileUpdateDto {
    pub provider_id: ProviderIdDto,
    pub info: ContentFileUpdateInfoDto,
}

impl From<ContentFile> for ContentFileDto {
    fn from(value: ContentFile) -> Self {
        Self {
            content_path: value.content_path,
            content_type: value.content_type.into(),
            disabled: value.disabled,
            file_name: value.file_name,
            hash: value.hash,
            name: value.name,
            size: value.size,
            version: value.version,
            update_provider_id: value.update_provider_id.map(Into::into),
            update: value.update.map(|updates| {
                let mut updates: Vec<ContentFileUpdateDto> = updates
                    .into_iter()
                    .map(|(provider_id, info)| ContentFileUpdateDto {
                        provider_id: provider_id.into(),
                        info: info.into(),
                    })
                    .collect();
                updates.sort_by(|a, b| {
                    (&a.provider_id.plugin_id, &a.provider_id.capability_id)
                        .cmp(&(&b.provider_id.plugin_id, &b.provider_id.capability_id))
                });
                updates
            }),
            metadata: value.metadata.map(|metadata| ContentMetadataDto {
                icon_path: value.icon_path.as_deref().map(format_asset_url),
                ..metadata.into()
            }),
            version_metadata: value.version_metadata.map(Into::into),
            version_metadata_pending: value.version_metadata_pending,
            metadata_pending: value.metadata_pending,
        }
    }
}

impl From<ContentFileUpdateInfo> for ContentFileUpdateInfoDto {
    fn from(value: ContentFileUpdateInfo) -> Self {
        Self {
            content_id: value.content_id,
            version: value.version,
        }
    }
}

impl From<ContentMetadata> for ContentMetadataDto {
    fn from(value: ContentMetadata) -> Self {
        Self {
            content_id: value.content_id,
            slug: value.slug,
            name: value.name,
            description: value.description,
            author: value.author,
            icon_url: value.icon_url,
            icon_path: None,
        }
    }
}

impl From<ContentVersionMetadata> for ContentVersionMetadataDto {
    fn from(value: ContentVersionMetadata) -> Self {
        Self {
            version_id: value.version_id,
            version_number: value.version_number,
            name: value.name,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use aether_core::features::instance::{ContentType, PackFile, ProviderId};

    use super::*;

    fn file(icon_path: Option<&str>) -> ContentFile {
        let mut file = ContentFile::from_pack_file(
            PackFile::from_hash("sodium.jar".into(), "hash".into()),
            "mods/sodium.jar".into(),
            ContentType::Mod,
            1,
            false,
        );
        file.metadata = Some(ContentMetadata {
            content_id: "AANobbMI".into(),
            slug: "sodium".into(),
            name: "Sodium".into(),
            description: None,
            author: None,
            author_ref: None,
            icon_url: Some("https://cdn.modrinth.com/data/AANobbMI/icon.png".into()),
        });
        file.icon_path = icon_path.map(Into::into);
        file
    }

    #[test]
    fn should_expose_the_cached_icon_as_an_asset_url_when_it_is_there() {
        let dto = ContentFileDto::from(file(Some("content-icons/abc.png")));

        let metadata = dto.metadata.unwrap();
        assert_eq!(
            metadata.icon_path,
            Some(format_asset_url("content-icons/abc.png"))
        );
        assert!(metadata.icon_url.is_some());
    }

    #[test]
    fn should_expose_no_icon_path_when_the_icon_has_not_arrived() {
        let dto = ContentFileDto::from(file(None));

        assert_eq!(dto.metadata.unwrap().icon_path, None);
    }

    #[test]
    fn should_expose_the_version_metadata_in_camel_case_when_cached() {
        let mut file = file(None);
        file.version_metadata = Some(ContentVersionMetadata {
            version_id: "IZskON6d".into(),
            version_number: "mc1.20.6-0.5.8".into(),
            name: Some("Sodium 0.5.8".into()),
        });

        let json = serde_json::to_value(ContentFileDto::from(file)).unwrap();

        assert_eq!(
            json["versionMetadata"],
            serde_json::json!({
                "versionId": "IZskON6d",
                "versionNumber": "mc1.20.6-0.5.8",
                "name": "Sodium 0.5.8",
            })
        );
    }

    #[test]
    fn should_expose_the_updates_as_entries_ordered_by_provider() {
        let provider = |plugin: &str, capability: &str| ProviderId {
            plugin_id: plugin.into(),
            capability_id: capability.into(),
        };
        let info = |content_id: &str| ContentFileUpdateInfo {
            content_id: content_id.into(),
            version: "1".into(),
        };
        let mut file = file(None);
        file.update = Some(HashMap::from([
            (provider("modrinth", "content"), info("b")),
            (provider("curseforge", "content"), info("a")),
        ]));

        let json = serde_json::to_value(ContentFileDto::from(file)).unwrap();

        assert_eq!(
            json["update"],
            serde_json::json!([
                {
                    "providerId": { "pluginId": "curseforge", "capabilityId": "content" },
                    "info": { "contentId": "a", "version": "1" },
                },
                {
                    "providerId": { "pluginId": "modrinth", "capabilityId": "content" },
                    "info": { "contentId": "b", "version": "1" },
                },
            ])
        );
    }

    #[test]
    fn should_expose_no_version_metadata_when_none_is_cached() {
        let json = serde_json::to_value(ContentFileDto::from(file(None))).unwrap();

        assert!(json["versionMetadata"].is_null());
    }
}
