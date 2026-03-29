use crate::features::{
    instance::{
        infra::content_providers::modrinth::{
            api_client::{Hit, ProjectSearchParams, ProjectSearchResponse, SearchIndex},
            get_facet_vector, ModrinthMapperError,
        },
        ContentItem, ContentSearchParams, ContentSearchResult, ContentType, ProviderId,
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

        if value.content_type == ContentType::Mod {
            if let Some(loader) = value.loader {
                if loader != ModLoader::Vanilla {
                    facets.push(get_facet_vector("categories", &[loader.as_str()]));
                }
            }
        }

        if let Some(game_versions) = value.game_versions {
            if !game_versions.is_empty() {
                facets.push(get_facet_vector("versions", &game_versions));
            }
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
            project_type: raw_type,
            author,
            icon_url,
            versions,
            ..
        } = value;

        let content_type = ContentType::from_string(&raw_type)
            .ok_or(ModrinthMapperError::UnknownProjectType(raw_type))?;

        let url = format!("https://modrinth.com/mod/{slug}");

        Ok(Self {
            id: project_id,
            slug,
            name: title,
            description: Some(description),
            content_type,
            url,
            author,
            icon_url,
            versions,
        })
    }
}
