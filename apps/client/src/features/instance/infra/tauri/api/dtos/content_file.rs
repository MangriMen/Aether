use std::collections::HashMap;

use aether_core::features::instance::{ContentFile, ContentFileUpdateInfo};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use specta::Type;

use crate::features::instance::{ContentTypeDto, ProviderIdDto};

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
    pub size: u64,
    pub version: Option<String>,
    pub update_provider_id: Option<ProviderIdDto>,
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    #[specta(type = Option<HashMap<String, ContentFileUpdateInfoDto>>)]
    pub update: Option<HashMap<ProviderIdDto, ContentFileUpdateInfoDto>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentFileUpdateInfoDto {
    pub content_id: String,
    pub version: String,
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
