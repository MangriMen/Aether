use crate::features::{
    instance::{
        ContentId, ContentItem, ContentMetadata, ContentSearchParams, ContentSearchResult,
        ContentType, ContentVersion, ContentVersionDependency, ContentVersionDependencyType,
        ContentVersionId, ContentVersionMetadata, ContentVersionStatus, ContentVersionType,
        PageIndex, PageSize, ProviderId, RequestedContentVersionStatus,
        infra::content_providers::modrinth::{
            ModrinthMapperError,
            api_client::{
                Hit, MODRINTH_WEB_URL, ModrinthDependencyResponse, ModrinthDependencyType,
                ModrinthRequestedStatus, ModrinthVersionStatus, ModrinthVersionType,
                ProjectMetadataResponse, ProjectResponse, ProjectSearchParams,
                ProjectSearchResponse, ProjectVersionResponse, SearchIndex,
                VersionMetadataResponse,
            },
        },
    },
    minecraft::{GameVersion, ModLoader},
};
use crate::shared::web_url::{HttpUrl, display_url};

/// The most results Modrinth serves in one search page.
const MODRINTH_MAX_PAGE_SIZE: u32 = 100;

/// How a Modrinth [`ContentMetadata::author_ref`] naming an organization by its id starts.
const ORGANIZATION_REF: &str = "organization:";

/// How a Modrinth [`ContentMetadata::author_ref`] naming a team by its id starts.
const TEAM_REF: &str = "team:";

/// The [`ContentMetadata::author_ref`] naming the organization `id`.
pub(super) fn organization_ref(id: &str) -> String {
    format!("{ORGANIZATION_REF}{id}")
}

/// The [`ContentMetadata::author_ref`] naming the team `id`.
pub(super) fn team_ref(id: &str) -> String {
    format!("{TEAM_REF}{id}")
}

/// What an [`ContentMetadata::author_ref`] of Modrinth names, by id.
pub(super) enum AuthorRef<'a> {
    Organization(&'a str),
    Team(&'a str),
}

impl<'a> AuthorRef<'a> {
    /// `None` for a ref of neither kind, or naming no id.
    pub(super) fn parse(author_ref: &'a str) -> Option<Self> {
        let (id, kind): (_, fn(&'a str) -> Self) =
            if let Some(id) = author_ref.strip_prefix(ORGANIZATION_REF) {
                (id, Self::Organization)
            } else {
                (author_ref.strip_prefix(TEAM_REF)?, Self::Team)
            };
        (!id.is_empty()).then(|| kind(id))
    }
}

/// Who authors a project, as its [`ContentMetadata::author_ref`]: the organization that owns it
/// if one does, else its team, whose owner Modrinth shows as the author then.
pub(super) fn author_ref(organization: Option<&str>, team: Option<&str>) -> Option<String> {
    let named = |id: &&str| !id.is_empty();
    organization
        .filter(named)
        .map(organization_ref)
        .or_else(|| team.filter(named).map(team_ref))
}

/// The page of a project or version on Modrinth's site. Only linked to, as the icons are only
/// shown: a bad one is dropped (see [`display_url`]) and costs the item nothing else.
fn page_url(what: &str, path: &str) -> Option<HttpUrl> {
    display_url(what, Some(format!("{MODRINTH_WEB_URL}/{path}")))
}

fn get_facet_vector<T: AsRef<str>>(facet: &str, values: &[T]) -> Vec<String> {
    values
        .iter()
        .map(|value| format!("{}:{}", facet, value.as_ref()))
        .collect()
}

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

        // Modrinth refuses a larger `limit`; the offset follows the size actually asked for, so the
        // pages stay adjacent.
        let page_size = value.page_size.at_most(MODRINTH_MAX_PAGE_SIZE);
        let offset = i64::try_from(value.page.offset(page_size)).map_err(|err| err.to_string())?;

        Ok(Self {
            index: SearchIndex::Relevance,
            offset,
            limit: page_size.into(),
            facets,
            query: value.query,
        })
    }
}

impl ProjectSearchResponse {
    pub(crate) fn into_content_search(
        self,
        provider_id: ProviderId,
    ) -> Result<ContentSearchResult, ModrinthMapperError> {
        let page_size = PageSize::parse(i64::from(self.limit))?;
        let (page, page_count) = calculate_pagination(self.offset, self.limit, self.total_hits);

        let items = self
            .hits
            .into_iter()
            .filter_map(|hit| hit.try_into().ok())
            .collect();

        Ok(ContentSearchResult {
            page: PageIndex::parse(i64::from(page))?,
            page_size,
            page_count,
            provider_id,
            items,
        })
    }
}

/// The 1-based page `offset` falls on, and how many pages `total_hits` fill. `limit` is not 0.
fn calculate_pagination(offset: u32, limit: u32, total_hits: u32) -> (u32, u32) {
    (offset / limit + 1, total_hits.div_ceil(limit))
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

        let url = page_url(
            "page of a Modrinth project",
            &format!("{project_type}/{slug}"),
        );

        Ok(Self {
            id: project_id,
            slug,
            name: title,
            description: Some(description),
            long_description: None,
            content_type,
            url,
            author,
            icon_url: display_url("icon of a Modrinth project", Some(icon_url)),
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

        let url = page_url(
            "page of a Modrinth project",
            &format!("{project_type}/{slug}"),
        );

        Ok(Self {
            id,
            slug,
            name: title,
            description: Some(description),
            long_description: Some(body),
            author: team,
            url,
            icon_url: display_url("icon of a Modrinth project", icon_url),
            versions,
            content_type,
        })
    }
}

impl From<&ProjectResponse> for ProjectMetadataResponse {
    /// What a full project says of itself, in the shape `ContentMetadata` is mapped from, so both
    /// answers choose the [`ContentMetadata::author_ref`] the same way.
    fn from(value: &ProjectResponse) -> Self {
        Self {
            id: value.id.clone(),
            slug: value.slug.clone(),
            title: value.title.clone(),
            icon_url: value.icon_url.clone(),
            team: Some(value.team.clone()),
            organization: value
                .organization
                .as_ref()
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
        }
    }
}

impl TryFrom<ProjectMetadataResponse> for ContentMetadata {
    type Error = ModrinthMapperError;

    /// No author: a project names only its team or organization by id, which its
    /// [`ContentMetadata::author_ref`] carries for the core to look up apart.
    fn try_from(value: ProjectMetadataResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            author_ref: author_ref(value.organization.as_deref(), value.team.as_deref()),
            content_id: ContentId::try_from(value.id)?,
            slug: value.slug,
            name: value.title,
            author: None,
            icon_url: display_url("icon of a Modrinth project", value.icon_url),
        })
    }
}

impl TryFrom<VersionMetadataResponse> for ContentVersionMetadata {
    type Error = ModrinthMapperError;

    fn try_from(value: VersionMetadataResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            version_id: ContentVersionId::try_from(value.id)?,
            version_number: value.version_number,
            name: value.name.filter(|name| !name.is_empty()),
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

impl TryFrom<ModrinthDependencyResponse> for ContentVersionDependency {
    type Error = ModrinthMapperError;

    fn try_from(value: ModrinthDependencyResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            version_id: value
                .version_id
                .map(ContentVersionId::try_from)
                .transpose()?,
            content_id: value.project_id.map(ContentId::try_from).transpose()?,
            file_name: value.file_name,
            dependency_type: match value.dependency_type {
                ModrinthDependencyType::Required => ContentVersionDependencyType::Required,
                ModrinthDependencyType::Optional => ContentVersionDependencyType::Optional,
                ModrinthDependencyType::Incompatible => ContentVersionDependencyType::Incompatible,
                ModrinthDependencyType::Embedded => ContentVersionDependencyType::Embedded,
            },
        })
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
            // A tag the provider invents that is not a loader of ours is dropped, not an error.
            .filter_map(|l| l.parse::<ModLoader>().ok())
            .collect();

        let id = ContentVersionId::try_from(id)?;
        let content_id = ContentId::try_from(project_id)?;
        let web_url = page_url(
            "page of a Modrinth version",
            &format!("project/{content_id}/version/{id}"),
        );

        Ok(Self {
            id,
            content_id,
            author_id,
            name,
            version_number,
            changelog,
            dependencies: dependencies
                .into_iter()
                .map(ContentVersionDependency::try_from)
                .collect::<Result<_, _>>()?,
            game_versions: GameVersion::parse_list(game_versions, "Modrinth version"),
            version_type: version_type.into(),
            loaders,
            featured,
            status: status.into(),
            requested_status: requested_status.map(Into::into),
            date_published,
            downloads: u64::try_from(downloads).unwrap_or_default(),
            web_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(offset: u32, limit: u32, total_hits: u32) -> ProjectSearchResponse {
        ProjectSearchResponse {
            hits: Vec::new(),
            offset,
            limit,
            total_hits,
        }
    }

    fn provider() -> ProviderId {
        ProviderId::from_parts("aether", "modrinth").unwrap()
    }

    #[test]
    fn should_count_the_page_and_the_pages_when_a_search_answers() {
        let result = response(40, 20, 95)
            .into_content_search(provider())
            .unwrap();

        assert_eq!(result.page, PageIndex::parse(3).unwrap());
        assert_eq!(result.page_size, PageSize::parse(20).unwrap());
        assert_eq!(result.page_count, 5);
    }

    #[test]
    fn should_count_no_pages_when_a_search_finds_nothing() {
        let result = response(0, 20, 0).into_content_search(provider()).unwrap();

        assert_eq!(result.page, PageIndex::FIRST);
        assert_eq!(result.page_count, 0);
    }

    #[test]
    fn should_refuse_the_answer_when_its_limit_is_zero() {
        let err = response(0, 0, 0)
            .into_content_search(provider())
            .unwrap_err();

        assert!(matches!(err, ModrinthMapperError::InvalidPagination(_)));
    }

    fn params(page: i64, page_size: i64) -> ContentSearchParams {
        ContentSearchParams {
            content_type: ContentType::Mod,
            provider_id: ProviderId::from_parts("aether", "modrinth").unwrap(),
            page: PageIndex::parse(page).unwrap(),
            page_size: PageSize::parse(page_size).unwrap(),
            query: None,
            game_versions: None,
            loader: None,
        }
    }

    #[test]
    fn should_skip_the_previous_pages_when_computing_the_search_offset() {
        let search = ProjectSearchParams::try_from(params(3, 20)).unwrap();

        assert_eq!(search.offset, 40);
        assert_eq!(search.limit, 20);
    }

    #[test]
    fn should_clamp_the_limit_and_follow_it_in_the_offset_when_the_page_size_is_over_modrinths_cap()
    {
        let search = ProjectSearchParams::try_from(params(3, 500)).unwrap();

        assert_eq!(search.limit, 100);
        assert_eq!(search.offset, 200);
    }
}
