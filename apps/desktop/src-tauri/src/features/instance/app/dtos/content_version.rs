use aether_core::features::instance::{
    app::ContentListVersionParams, ContentVersion, ContentVersionDependency,
    ContentVersionDependencyType, ContentVersionStatus, ContentVersionType,
    RequestedContentVersionStatus,
};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::features::{instance::ProviderIdDto, minecraft::ModLoaderDto};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentListVersionParamsDto {
    pub content_id: String,
    pub provider_id: ProviderIdDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "lowercase")]
pub enum ContentVersionDependencyTypeDto {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ContentVersionDependencyDto {
    pub version_id: Option<String>,
    pub content_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: ContentVersionDependencyTypeDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "lowercase")]
pub enum ContentVersionTypeDto {
    Release,
    Beta,
    Alpha,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "lowercase")]
pub enum ContentVersionStatusDto {
    Listed,
    Archived,
    Draft,
    Unlisted,
    Scheduled,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "lowercase")]
pub enum RequestedContentVersionStatusDto {
    Listed,
    Archived,
    Draft,
    Unlisted,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ContentVersionDto {
    pub id: String,
    pub content_id: String,
    pub author_id: String,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub dependencies: Vec<ContentVersionDependencyDto>,
    pub game_versions: Vec<String>,
    pub version_type: ContentVersionTypeDto,
    pub loaders: Vec<ModLoaderDto>,
    pub featured: bool,
    pub status: ContentVersionStatusDto,
    pub requested_status: Option<RequestedContentVersionStatusDto>,
    pub date_published: String,
    pub downloads: i64,
    pub web_url: String,
}

impl From<ContentListVersionParamsDto> for ContentListVersionParams {
    fn from(value: ContentListVersionParamsDto) -> Self {
        Self {
            content_id: value.content_id,
            provider_id: value.provider_id.into(),
        }
    }
}

impl From<ContentVersionDependencyType> for ContentVersionDependencyTypeDto {
    fn from(value: ContentVersionDependencyType) -> Self {
        match value {
            ContentVersionDependencyType::Required => Self::Required,
            ContentVersionDependencyType::Optional => Self::Optional,
            ContentVersionDependencyType::Incompatible => Self::Incompatible,
            ContentVersionDependencyType::Embedded => Self::Embedded,
        }
    }
}

impl From<ContentVersionDependency> for ContentVersionDependencyDto {
    fn from(value: ContentVersionDependency) -> Self {
        Self {
            version_id: value.version_id,
            content_id: value.content_id,
            file_name: value.file_name,
            dependency_type: value.dependency_type.into(),
        }
    }
}

impl From<ContentVersionType> for ContentVersionTypeDto {
    fn from(value: ContentVersionType) -> Self {
        match value {
            ContentVersionType::Release => Self::Release,
            ContentVersionType::Beta => Self::Beta,
            ContentVersionType::Alpha => Self::Alpha,
        }
    }
}

impl From<ContentVersionStatus> for ContentVersionStatusDto {
    fn from(value: ContentVersionStatus) -> Self {
        match value {
            ContentVersionStatus::Listed => Self::Listed,
            ContentVersionStatus::Archived => Self::Archived,
            ContentVersionStatus::Draft => Self::Draft,
            ContentVersionStatus::Unlisted => Self::Unlisted,
            ContentVersionStatus::Scheduled => Self::Scheduled,
            ContentVersionStatus::Unknown => Self::Unknown,
        }
    }
}

impl From<RequestedContentVersionStatus> for RequestedContentVersionStatusDto {
    fn from(value: RequestedContentVersionStatus) -> Self {
        match value {
            RequestedContentVersionStatus::Listed => Self::Listed,
            RequestedContentVersionStatus::Archived => Self::Archived,
            RequestedContentVersionStatus::Draft => Self::Draft,
            RequestedContentVersionStatus::Unlisted => Self::Unlisted,
        }
    }
}

impl From<ContentVersion> for ContentVersionDto {
    fn from(value: ContentVersion) -> Self {
        Self {
            id: value.id,
            content_id: value.content_id,
            author_id: value.author_id,
            name: value.name,
            version_number: value.version_number,
            changelog: value.changelog,
            dependencies: value.dependencies.into_iter().map(Into::into).collect(),
            game_versions: value.game_versions,
            version_type: value.version_type.into(),
            loaders: value.loaders.into_iter().map(Into::into).collect(),
            featured: value.featured,
            status: value.status.into(),
            requested_status: value.requested_status.map(Into::into),
            date_published: value.date_published,
            downloads: value.downloads,
            web_url: value.web_url,
        }
    }
}
