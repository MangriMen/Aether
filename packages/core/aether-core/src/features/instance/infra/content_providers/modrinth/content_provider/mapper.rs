use std::str::FromStr;

use crate::features::{
    instance::{
        ContentItem, ContentSearchParams, ContentSearchResult, ContentType, ContentVersion,
        ContentVersionDependency, ContentVersionDependencyType, ContentVersionStatus,
        ContentVersionType, ProviderId, RequestedContentVersionStatus,
        infra::content_providers::modrinth::{
            ModrinthMapperError,
            api_client::{
                Hit, MODRINTH_WEB_URL, ModrinthDependencyResponse, ModrinthDependencyType,
                ModrinthRequestedStatus, ModrinthVersionStatus, ModrinthVersionType,
                ProjectResponse, ProjectSearchParams, ProjectSearchResponse,
                ProjectVersionResponse, SearchIndex,
            },
            get_facet_vector,
        },
    },
    minecraft::ModLoader,
};

impl TryFrom<ContentSearchParams> for ProjectSearchParams {
    type Error = String;

    fn try_from(value: ContentSearchParams) -> Result<Self, Self::Error> {
        let mut facets = Vec::with_capacity(3);

        facets.push(get_facet_vector(
            "project_type",
            &[value.content_type.get_name()],
        ));

        if value.content_type == ContentType::Mod
            && let Some(loader) = value.loader
            && loader != ModLoader::Vanilla
        {
            facets.push(get_facet_vector("categories", &[loader.as_str()]));
        }

        if let Some(game_versions) = value.game_versions
            && !game_versions.is_empty()
        {
            facets.push(get_facet_vector("versions", &game_versions));
        }

        Ok(Self {
            index: SearchIndex::Relevance,
            offset: (value.page - 1) * value.page_size,
            limit: value.page_size,
            facets,
            query: value.query,
        })
    }
}

impl ProjectSearchResponse {
    pub fn into_content_search(self, provider_id: ProviderId) -> ContentSearchResult {
        let (page, page_count) = calculate_pagination(self.offset, self.limit, self.total_hits);

        let items = self
            .hits
            .into_iter()
            .filter_map(|hit| hit.try_into().ok())
            .collect();

        ContentSearchResult {
            page,
            page_size: self.limit,
            page_count,
            provider_id,
            items,
        }
    }
}

fn calculate_pagination(offset: i64, limit: i64, total_hits: i64) -> (i64, i64) {
    if limit <= 0 {
        return (1, 0);
    }

    let page = (offset / limit) + 1;
    let page_count = (total_hits as f64 / limit as f64).ceil() as i64;

    (page, page_count)
}

impl TryFrom<Hit> for ContentItem {
    type Error = ModrinthMapperError;

    fn try_from(value: Hit) -> Result<Self, Self::Error> {
        let Hit {
            project_id,
            slug,
            title,
            description,
            project_type,
            author,
            icon_url,
            versions,
            ..
        } = value;

        let content_type = ContentType::from_string(&project_type).ok_or(
            ModrinthMapperError::UnknownProjectType(project_type.clone()),
        )?;

        let url = format!("{}/{project_type}/{slug}", MODRINTH_WEB_URL);

        Ok(Self {
            id: project_id,
            slug,
            name: title,
            description: Some(description),
            long_description: None,
            content_type,
            url,
            author,
            icon_url,
            versions,
        })
    }
}

impl TryFrom<ProjectResponse> for ContentItem {
    type Error = ModrinthMapperError;

    fn try_from(value: ProjectResponse) -> Result<Self, Self::Error> {
        let ProjectResponse {
            id,
            slug,
            title,
            description,
            project_type,
            team,
            icon_url,
            versions,
            body,
            ..
        } = value;

        let content_type = ContentType::from_string(&project_type).ok_or(
            ModrinthMapperError::UnknownProjectType(project_type.clone()),
        )?;

        let url = format!("{}/{project_type}/{slug}", MODRINTH_WEB_URL);

        Ok(Self {
            id,
            slug,
            name: title,
            description: Some(description),
            long_description: Some(body),
            author: team,
            url,
            icon_url: icon_url.unwrap_or_default(),
            versions,
            content_type,
        })
    }
}

impl From<ModrinthVersionType> for ContentVersionType {
    fn from(value: ModrinthVersionType) -> Self {
        match value {
            ModrinthVersionType::Release => Self::Release,
            ModrinthVersionType::Beta => Self::Beta,
            ModrinthVersionType::Alpha => Self::Alpha,
        }
    }
}

impl From<ModrinthVersionStatus> for ContentVersionStatus {
    fn from(value: ModrinthVersionStatus) -> Self {
        match value {
            ModrinthVersionStatus::Listed => Self::Listed,
            ModrinthVersionStatus::Archived => Self::Archived,
            ModrinthVersionStatus::Draft => Self::Draft,
            ModrinthVersionStatus::Unlisted => Self::Unlisted,
            ModrinthVersionStatus::Scheduled => Self::Scheduled,
        }
    }
}

impl From<ModrinthRequestedStatus> for RequestedContentVersionStatus {
    fn from(value: ModrinthRequestedStatus) -> Self {
        match value {
            ModrinthRequestedStatus::Listed => Self::Listed,
            ModrinthRequestedStatus::Archived => Self::Archived,
            ModrinthRequestedStatus::Draft => Self::Draft,
            ModrinthRequestedStatus::Unlisted => Self::Unlisted,
        }
    }
}

impl From<ModrinthDependencyResponse> for ContentVersionDependency {
    fn from(value: ModrinthDependencyResponse) -> Self {
        Self {
            version_id: value.version_id,
            content_id: value.project_id,
            file_name: value.file_name,
            dependency_type: match value.dependency_type {
                ModrinthDependencyType::Required => ContentVersionDependencyType::Required,
                ModrinthDependencyType::Optional => ContentVersionDependencyType::Optional,
                ModrinthDependencyType::Incompatible => ContentVersionDependencyType::Incompatible,
                ModrinthDependencyType::Embedded => ContentVersionDependencyType::Embedded,
            },
        }
    }
}

impl TryFrom<ProjectVersionResponse> for ContentVersion {
    type Error = ModrinthMapperError;

    fn try_from(value: ProjectVersionResponse) -> Result<Self, Self::Error> {
        let ProjectVersionResponse {
            id,
            project_id,
            author_id,
            name,
            changelog,
            game_versions,
            featured,
            date_published,
            downloads,
            version_number,
            version_type,
            status,
            requested_status,
            loaders,
            dependencies,
            ..
        } = value;

        let loaders = loaders
            .into_iter()
            .filter_map(|l| ModLoader::from_str(&l).ok())
            .collect();

        let web_url = format!("{}/project/{}/version/{}", MODRINTH_WEB_URL, project_id, id);

        Ok(Self {
            id,
            content_id: project_id,
            author_id,
            name,
            version_number,
            changelog,
            dependencies: dependencies.into_iter().map(Into::into).collect(),
            game_versions,
            version_type: version_type.into(),
            loaders,
            featured,
            status: status.into(),
            requested_status: requested_status.map(Into::into),
            date_published,
            downloads,
            web_url,
        })
    }
}
