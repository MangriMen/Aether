use std::{collections::HashMap, path::Path, sync::Arc};

use async_trait::async_trait;

use crate::{
    features::{
        instance::{
            AtomicInstallParams, CapabilityMetadata, ContentFile, ContentItem, ContentProvider,
            ContentProviderCapabilityMetadata, ContentSearchParams, ContentSearchResult,
            ContentVersion, CreateContentFileParams, DownloadedContent, Instance, InstanceError,
            ProviderId,
            app::{ContentCompatibilityCheckParams, ContentCompatibilityResult},
            infra::content_providers::modrinth::{
                ModrinthMapperError,
                api_client::{
                    File, MODRINTH_API_URL, ModrinthApiClient, ProjectSearchParams,
                    ProjectVersionResponse, ProjectVersionsRequest,
                },
            },
        },
        settings::LocationInfo,
    },
    shared::io::infra::write_async,
    shared::request_client::{Request, RequestClient},
};

use super::{find_best_version, get_first_file_from_project_version, is_version_compatible};

pub struct ModrinthContentProvider<RC> {
    location_info: Arc<LocationInfo>,
    request_client: Arc<RC>,
    api: ModrinthApiClient<RC>,
    capability: ContentProviderCapabilityMetadata,
}

impl ModrinthContentProvider<()> {
    pub const ID: &'static str = "core:modrinth";
}

impl<RC: RequestClient> ModrinthContentProvider<RC> {
    pub fn new(
        location_info: Arc<LocationInfo>,
        base_headers: Option<reqwest::header::HeaderMap>,
        request_client: Arc<RC>,
    ) -> Self {
        let capability = ContentProviderCapabilityMetadata {
            base: CapabilityMetadata {
                id: "modrinth-content".to_string(),
                name: "Modrinth".to_string(),
                description: None,
                icon: None,
            },
            supports_install_atomic: true,
            supports_install_modpacks: false,
        };

        Self {
            request_client: request_client.clone(),
            api: ModrinthApiClient::new(MODRINTH_API_URL.to_string(), base_headers, request_client),
            location_info,
            capability,
        }
    }

    fn get_provider_id(&self) -> ProviderId {
        ProviderId {
            plugin_id: ModrinthContentProvider::ID.to_owned(),
            capability_id: self.capability.base.id.clone(),
        }
    }

    pub async fn get_project_version_for_game_version(
        &self,
        project_id: &str,
        game_version: &str,
        loader: Option<&String>,
    ) -> Result<ProjectVersionResponse, InstanceError> {
        let request_params = ProjectVersionsRequest {
            loaders: loader.map(|l| vec![l.clone()]),
            game_versions: Some(vec![game_version.to_string()]),
            include_changelog: false,
            ..Default::default()
        };

        let versions = self
            .api
            .get_project_versions(project_id, &request_params)
            .await
            .map_err(InstanceError::ContentDownloadError)?;

        find_best_version(&versions, game_version, loader)
            .cloned()
            .ok_or_else(|| InstanceError::ContentForGameVersionNotFound {
                game_version: game_version.to_owned(),
            })
    }

    async fn resolve_project_version(
        &self,
        install_params: &AtomicInstallParams,
    ) -> Result<ProjectVersionResponse, InstanceError> {
        match &install_params.content_version {
            Some(version) => self
                .api
                .get_project_version(version)
                .await
                .map_err(|err| InstanceError::ContentDownloadError(err.clone())),
            None => {
                self.get_project_version_for_game_version(
                    &install_params.content_id,
                    &install_params.game_version,
                    install_params.loader.as_ref(),
                )
                .await
            }
        }
    }

    fn get_project_file<'a>(
        project_version: &'a ProjectVersionResponse,
        install_params: &AtomicInstallParams,
    ) -> Result<&'a File, InstanceError> {
        get_first_file_from_project_version(project_version).ok_or(
            InstanceError::ContentForGameVersionNotFound {
                game_version: install_params.game_version.clone(),
            },
        )
    }

    async fn fetch_to_disk(&self, url: &str, path: &Path) -> Result<(), InstanceError> {
        let file_bytes = self
            .request_client
            .fetch_bytes(Request::get(url))
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))?;

        write_async(path, &file_bytes)
            .await
            .map_err(|err| InstanceError::Storage(err.to_string()))
    }
}

#[async_trait]
impl<RC: RequestClient> ContentProvider for ModrinthContentProvider<RC> {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata {
        &self.capability
    }

    async fn search(
        &self,
        search_params: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        let provider_id = search_params.provider_id.clone();

        let modrinth_search_params =
            ProjectSearchParams::try_from(search_params).map_err(|err| {
                InstanceError::ContentProviderError {
                    reason: format!("Invalid search params: {err}"),
                }
            })?;

        let response = self
            .api
            .search(&modrinth_search_params)
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.clone()))?;

        Ok(response.into_content_search(provider_id))
    }

    async fn install_atomic(
        &self,
        install_params: &AtomicInstallParams,
    ) -> Result<DownloadedContent, InstanceError> {
        let project_version = self.resolve_project_version(install_params).await?;

        let file = Self::get_project_file(&project_version, install_params)?;

        let content_path = install_params
            .content_type
            .get_relative_path(&file.filename);

        let temp_path = self.location_info.temp_dir().join(format!(
            "{}-{}",
            file.filename,
            uuid::Uuid::new_v4()
        ));

        self.fetch_to_disk(&file.url, &temp_path).await?;

        let metadata = ContentFile::from_params(CreateContentFileParams {
            name: Some(project_version.name.clone()),
            file_name: file.filename.clone(),
            size: file.size.max(0).cast_unsigned(),
            sha1: file.hashes.sha1.clone(),
            content_path,
            content_id: install_params.content_id.clone(),
            content_version: project_version.id,
            content_type: install_params.content_type,
            provider_id: self.get_provider_id(),
        });

        Ok(DownloadedContent {
            metadata,
            temp_path,
        })
    }

    async fn check_compatibility(
        &self,
        instances: &[Instance],
        check_params: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError> {
        let project = self
            .api
            .get_project(&check_params.content_item.slug)
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.clone()))?;

        let project_versions = self
            .api
            .get_project_versions(&project.id, &ProjectVersionsRequest::without_changelog())
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.clone()))?;

        let mut compatibility_map = HashMap::new();

        for instance in instances {
            let is_compatible = project_versions
                .iter()
                .any(|version| is_version_compatible(version, &project, instance));

            compatibility_map.insert(
                instance.id().to_owned(),
                ContentCompatibilityResult { is_compatible },
            );
        }

        Ok(compatibility_map)
    }

    async fn get_content(&self, content_id: String) -> Result<ContentItem, InstanceError> {
        self.api
            .get_project(&content_id)
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.clone()))?
            .try_into()
            .map_err(|err: ModrinthMapperError| {
                InstanceError::ContentDownloadError(err.to_string())
            })
    }

    async fn list_versions(
        &self,
        content_id: String,
    ) -> Result<Vec<ContentVersion>, InstanceError> {
        let versions = self
            .api
            .get_project_versions(&content_id, &ProjectVersionsRequest::without_changelog())
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.clone()))?;

        Ok(versions
            .into_iter()
            .filter_map(|v| v.try_into().ok()) // Skip versions that failed to map
            .collect())
    }
}
