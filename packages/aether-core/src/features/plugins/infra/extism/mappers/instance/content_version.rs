use aether_core_plugin_api::v0::{
    ContentVersionDependencyDto, ContentVersionDependencyTypeDto, ContentVersionDto,
    ContentVersionStatusDto, ContentVersionTypeDto, ModLoaderDto, RequestedContentVersionStatusDto,
};

use crate::features::instance::{
    ContentVersion, ContentVersionDependency, ContentVersionDependencyType, ContentVersionStatus,
    ContentVersionType, RequestedContentVersionStatus,
};
use crate::features::minecraft::ModLoader;

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
            loaders: value.loaders.into_iter().map(ModLoaderDto::from).collect(),
            featured: value.featured,
            status: value.status.into(),
            requested_status: value.requested_status.map(Into::into),
            date_published: value.date_published,
            downloads: value.downloads,
            web_url: value.web_url,
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

impl From<ContentVersionDto> for ContentVersion {
    fn from(value: ContentVersionDto) -> Self {
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
            loaders: value.loaders.into_iter().map(ModLoader::from).collect(),
            featured: value.featured,
            status: value.status.into(),
            requested_status: value.requested_status.map(Into::into),
            date_published: value.date_published,
            downloads: value.downloads,
            web_url: value.web_url,
        }
    }
}

impl From<ContentVersionDependencyDto> for ContentVersionDependency {
    fn from(value: ContentVersionDependencyDto) -> Self {
        Self {
            version_id: value.version_id,
            content_id: value.content_id,
            file_name: value.file_name,
            dependency_type: value.dependency_type.into(),
        }
    }
}

impl From<ContentVersionDependencyTypeDto> for ContentVersionDependencyType {
    fn from(value: ContentVersionDependencyTypeDto) -> Self {
        match value {
            ContentVersionDependencyTypeDto::Required => Self::Required,
            ContentVersionDependencyTypeDto::Optional => Self::Optional,
            ContentVersionDependencyTypeDto::Incompatible => Self::Incompatible,
            ContentVersionDependencyTypeDto::Embedded => Self::Embedded,
        }
    }
}

impl From<ContentVersionTypeDto> for ContentVersionType {
    fn from(value: ContentVersionTypeDto) -> Self {
        match value {
            ContentVersionTypeDto::Release => Self::Release,
            ContentVersionTypeDto::Beta => Self::Beta,
            ContentVersionTypeDto::Alpha => Self::Alpha,
        }
    }
}

impl From<ContentVersionStatusDto> for ContentVersionStatus {
    fn from(value: ContentVersionStatusDto) -> Self {
        match value {
            ContentVersionStatusDto::Listed => Self::Listed,
            ContentVersionStatusDto::Archived => Self::Archived,
            ContentVersionStatusDto::Draft => Self::Draft,
            ContentVersionStatusDto::Unlisted => Self::Unlisted,
            ContentVersionStatusDto::Scheduled => Self::Scheduled,
            ContentVersionStatusDto::Unknown => Self::Unknown,
        }
    }
}

impl From<RequestedContentVersionStatusDto> for RequestedContentVersionStatus {
    fn from(value: RequestedContentVersionStatusDto) -> Self {
        match value {
            RequestedContentVersionStatusDto::Listed => Self::Listed,
            RequestedContentVersionStatusDto::Archived => Self::Archived,
            RequestedContentVersionStatusDto::Draft => Self::Draft,
            RequestedContentVersionStatusDto::Unlisted => Self::Unlisted,
        }
    }
}
