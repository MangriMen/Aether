use aether_core_plugin_api::v0::{ContentFileDto, ContentFileUpdateInfoDto, ProviderIdDto};

use crate::features::instance::{
    ContentFile, ContentFileUpdateInfo, ContentPath, InvalidContentId, InvalidId,
    InvalidRelativePath, ProviderId,
};

impl From<ContentFile> for ContentFileDto {
    fn from(value: ContentFile) -> Self {
        Self {
            content_path: value.content_path.into_string(),
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
                        content_id: info.content_id.into(),
                        version: info.version.into(),
                    })
                    .collect()
            }),
        }
    }
}

/// What is wrong with a content file a plugin handed over.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidContentFile {
    #[error(transparent)]
    Id(#[from] InvalidId),

    #[error(transparent)]
    ContentId(#[from] InvalidContentId),

    #[error(transparent)]
    Path(#[from] InvalidRelativePath),
}

impl TryFrom<ContentFileDto> for ContentFile {
    type Error = InvalidContentFile;

    fn try_from(value: ContentFileDto) -> Result<Self, Self::Error> {
        Ok(Self {
            content_path: ContentPath::parse(&value.content_path)?,
            content_type: value.content_type.into(),
            disabled: value.disabled,
            file_name: value.file_name,
            hash: value.hash,
            name: value.name,
            size: value.size,
            version: None,
            update_provider_id: value
                .update_provider_id
                .map(ProviderId::try_from)
                .transpose()?,
            update: value
                .update
                .map(|update| {
                    update
                        .into_iter()
                        .map(|info| {
                            Ok((
                                info.provider_id.try_into()?,
                                ContentFileUpdateInfo {
                                    content_id: info.content_id.try_into()?,
                                    version: info.version.try_into()?,
                                },
                            ))
                        })
                        .collect::<Result<_, InvalidContentFile>>()
                })
                .transpose()?,
        })
    }
}

impl From<ProviderId> for ProviderIdDto {
    fn from(value: ProviderId) -> Self {
        Self {
            plugin_id: value.plugin_id.into(),
            capability_id: value.capability_id.into(),
        }
    }
}

impl TryFrom<ProviderIdDto> for ProviderId {
    type Error = InvalidId;

    fn try_from(value: ProviderIdDto) -> Result<Self, Self::Error> {
        Self::from_parts(&value.plugin_id, &value.capability_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::features::instance::{ContentId, ContentVersionId};
    use std::collections::HashMap;

    use extism_convert::{FromBytes, Msgpack, ToBytes};

    use super::*;
    use crate::features::instance::ContentType;

    /// Update info keyed by provider id crosses the plugin boundary both ways.
    #[test]
    fn should_round_trip_update_info_through_msgpack_when_encoded_and_decoded() {
        let modrinth: ProviderId = "aether#modrinth".parse().unwrap();
        let file = ContentFile {
            content_path: ContentPath::parse("mods/sodium.jar").unwrap(),
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
                    content_id: ContentId::try_from("AANobbMI").unwrap(),
                    version: ContentVersionId::try_from("IZskON6d").unwrap(),
                },
            )])),
        };

        let bytes = Msgpack(ContentFileDto::from(file))
            .to_bytes()
            .expect("DTO should encode");
        let Msgpack(dto) = Msgpack::<ContentFileDto>::from_bytes(&bytes).expect("should decode");
        let back = ContentFile::try_from(dto).expect("ids should parse");

        assert_eq!(back.update_provider_id, Some(modrinth.clone()));
        let info = &back.update.expect("update info should survive")[&modrinth];
        assert_eq!(info.content_id, ContentId::try_from("AANobbMI").unwrap());
        assert_eq!(
            info.version,
            ContentVersionId::try_from("IZskON6d").unwrap()
        );
    }

    #[test]
    fn should_refuse_a_provider_id_dto_when_a_part_is_not_an_id() {
        let err = ProviderId::try_from(ProviderIdDto {
            plugin_id: "core:modrinth".into(),
            capability_id: "modrinth-content".into(),
        })
        .unwrap_err();

        assert_eq!(err.kind, "plugin");
        assert_eq!(err.value, "core:modrinth");
    }
}
