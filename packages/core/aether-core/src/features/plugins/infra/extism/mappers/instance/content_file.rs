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
            update: value
                .update
                .map(|m| m.into_iter().map(|(k, v)| (k.into(), v.into())).collect()),
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
