use aether_core_plugin_api::v0::{ContentFileDto, ContentFileUpdateInfoDto, ProviderIdDto};

use crate::features::instance::{ContentFile, ContentFileUpdateInfo, ProviderId};

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
            update_provider_id: value.update_provider_id.map(Into::into),
            update: value.update.map(|update| {
                update
                    .into_iter()
                    .map(|(provider_id, info)| ContentFileUpdateInfoDto {
                        provider_id: provider_id.into(),
                        content_id: info.content_id,
                        version: info.version,
                    })
                    .collect()
            }),
        }
    }
}

impl From<ContentFileDto> for ContentFile {
    fn from(value: ContentFileDto) -> Self {
        Self {
            content_path: value.content_path,
            content_type: value.content_type.into(),
            disabled: value.disabled,
            file_name: value.file_name,
            hash: value.hash,
            name: value.name,
            size: value.size,
            version: None,
            update_provider_id: value.update_provider_id.map(Into::into),
            metadata: None,
            update: value.update.map(|update| {
                update
                    .into_iter()
                    .map(|info| {
                        (
                            info.provider_id.into(),
                            ContentFileUpdateInfo {
                                content_id: info.content_id,
                                version: info.version,
                            },
                        )
                    })
                    .collect()
            }),
        }
    }
}

impl From<ProviderId> for ProviderIdDto {
    fn from(value: ProviderId) -> Self {
        Self {
            plugin_id: value.plugin_id,
            capability_id: value.capability_id,
        }
    }
}

impl From<ProviderIdDto> for ProviderId {
    fn from(value: ProviderIdDto) -> Self {
        Self {
            plugin_id: value.plugin_id,
            capability_id: value.capability_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use extism_convert::{FromBytes, Msgpack, ToBytes};

    use super::*;
    use crate::features::instance::ContentType;

    /// Update info under a provider id containing a `:` (the old built-in Modrinth id, still in
    /// stored data) crosses the plugin boundary both ways.
    #[test]
    fn should_round_trip_update_info_through_msgpack_when_encoded_and_decoded() {
        let modrinth: ProviderId = "core:modrinth#modrinth-content".parse().unwrap();
        let file = ContentFile {
            content_path: "mods/sodium.jar".into(),
            content_type: ContentType::Mod,
            disabled: false,
            file_name: "sodium.jar".into(),
            hash: "123".into(),
            name: None,
            size: 1024,
            version: None,
            update_provider_id: Some(modrinth.clone()),
            update: Some(HashMap::from([(
                modrinth.clone(),
                ContentFileUpdateInfo {
                    content_id: "AANobbMI".into(),
                    version: "IZskON6d".into(),
                },
            )])),
            metadata: None,
        };

        let bytes = Msgpack(ContentFileDto::from(file))
            .to_bytes()
            .expect("DTO should encode");
        let Msgpack(dto) = Msgpack::<ContentFileDto>::from_bytes(&bytes).expect("should decode");
        let back = ContentFile::from(dto);

        assert_eq!(back.update_provider_id, Some(modrinth.clone()));
        let info = &back.update.expect("update info should survive")[&modrinth];
        assert_eq!(info.content_id, "AANobbMI");
        assert_eq!(info.version, "IZskON6d");
    }
}
