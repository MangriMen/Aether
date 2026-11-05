use aether_core_plugin_api::v0::{
    ContentVersionDependencyDto, ContentVersionDependencyTypeDto, ContentVersionDto,
    ContentVersionStatusDto, ContentVersionTypeDto, ModLoaderDto, RequestedContentVersionStatusDto,
};

use crate::features::instance::{
    ContentVersion, ContentVersionDependency, ContentVersionDependencyType, ContentVersionStatus,
    ContentVersionType, InvalidContentId, RequestedContentVersionStatus,
};
use crate::features::minecraft::{GameVersion, ModLoader};

use crate::shared::web_url::display_url;

impl From<ContentVersion> for ContentVersionDto {
    fn from(value: ContentVersion) -> Self {
        Self {
            id: value.id.into(),
            content_id: value.content_id.into(),
            author_id: value.author_id,
            name: value.name,
            version_number: value.version_number,
            changelog: value.changelog,
            dependencies: value.dependencies.into_iter().map(Into::into).collect(),
            game_versions: value.game_versions.into_iter().map(String::from).collect(),
            version_type: value.version_type.into(),
            loaders: value.loaders.into_iter().map(ModLoaderDto::from).collect(),
            featured: value.featured,
            status: value.status.into(),
            requested_status: value.requested_status.map(Into::into),
            date_published: value.date_published,
            downloads: i64::try_from(value.downloads).unwrap_or(i64::MAX),
            web_url: value.web_url.map(String::from).unwrap_or_default(),
        }
    }
}

impl From<ContentVersionDependency> for ContentVersionDependencyDto {
    fn from(value: ContentVersionDependency) -> Self {
        Self {
            version_id: value.version_id.map(Into::into),
            content_id: value.content_id.map(Into::into),
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

impl TryFrom<ContentVersionDto> for ContentVersion {
    type Error = InvalidContentId;

    fn try_from(value: ContentVersionDto) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.try_into()?,
            content_id: value.content_id.try_into()?,
            author_id: value.author_id,
            name: value.name,
            version_number: value.version_number,
            changelog: value.changelog,
            dependencies: value
                .dependencies
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            game_versions: GameVersion::parse_list(value.game_versions, "plugin content version"),
            version_type: value.version_type.into(),
            loaders: value.loaders.into_iter().map(ModLoader::from).collect(),
            featured: value.featured,
            status: value.status.into(),
            requested_status: value.requested_status.map(Into::into),
            date_published: value.date_published,
            downloads: u64::try_from(value.downloads).unwrap_or_default(),
            web_url: display_url("page of a plugin's content version", Some(value.web_url)),
        })
    }
}

impl TryFrom<ContentVersionDependencyDto> for ContentVersionDependency {
    type Error = InvalidContentId;

    fn try_from(value: ContentVersionDependencyDto) -> Result<Self, Self::Error> {
        Ok(Self {
            version_id: value.version_id.map(TryInto::try_into).transpose()?,
            content_id: value.content_id.map(TryInto::try_into).transpose()?,
            file_name: value.file_name,
            dependency_type: value.dependency_type.into(),
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_drop_only_the_invalid_game_versions_when_a_plugin_lists_them() {
        let dto = ContentVersionDto {
            id: "IZskON6d".into(),
            content_id: "AANobbMI".into(),
            author_id: "author".into(),
            name: "Sodium".into(),
            version_number: "0.6.0".into(),
            changelog: None,
            dependencies: Vec::new(),
            game_versions: vec!["1.21.1".into(), "../x".into(), "1.21.4".into()],
            version_type: ContentVersionTypeDto::Release,
            loaders: Vec::new(),
            featured: false,
            status: ContentVersionStatusDto::Listed,
            requested_status: None,
            date_published: "2024-01-01T00:00:00Z".into(),
            downloads: 0,
            web_url: String::new(),
        };

        let version = ContentVersion::try_from(dto).unwrap();

        assert_eq!(version.game_versions, ["1.21.1", "1.21.4"]);
    }
}
