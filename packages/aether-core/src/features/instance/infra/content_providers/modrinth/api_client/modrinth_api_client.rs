use crate::shared::request_client::{Request, RequestClient, RequestClientExt};
use std::sync::Arc;

use super::models::{
    Lenient, OrganizationResponse, ProjectMetadataResponse, ProjectResponse, ProjectSearchParams,
    ProjectSearchResponse, ProjectVersionResponse, ProjectVersionsRequest, TeamMemberResponse,
    VersionMetadataResponse,
};

pub(crate) struct ModrinthApiClient<RC> {
    base_url: String,
    /// The base of version 3 of the API, for the few routes version 2 lacks.
    v3_base_url: String,
    base_headers: Option<reqwest::header::HeaderMap>,
    request_client: Arc<RC>,
}

impl<RC: RequestClient> ModrinthApiClient<RC> {
    pub(crate) fn new(
        base_url: String,
        v3_base_url: String,
        base_headers: Option<reqwest::header::HeaderMap>,
        request_client: Arc<RC>,
    ) -> Self {
        Self {
            base_url,
            v3_base_url,
            base_headers,
            request_client,
        }
    }

    fn prepare_request(&self, path: &str, query: Option<String>) -> Request {
        self.prepare_request_at(&self.base_url, path, query)
    }

    fn prepare_request_at(&self, base: &str, path: &str, query: Option<String>) -> Request {
        let base = base.trim_end_matches('/');
        let path = path.trim_start_matches('/');

        let url = match query {
            Some(q) => format!("{base}/{path}?{q}"),
            None => format!("{base}/{path}"),
        };

        let mut request = Request::get(&url);
        if let Some(headers) = &self.base_headers {
            request = request.with_headers(headers.clone());
        }
        request
    }

    async fn fetch<T: serde::de::DeserializeOwned>(&self, request: Request) -> Result<T, String> {
        self.request_client
            .fetch_json_with_progress(request, None)
            .await
            .map_err(|err| err.to_string())
    }

    pub(crate) async fn search(
        &self,
        params: &ProjectSearchParams,
    ) -> Result<ProjectSearchResponse, String> {
        let query = serde_qs::to_string(params).map_err(|e| e.to_string())?;
        let request = self.prepare_request("search", Some(query));
        self.fetch(request).await
    }

    pub(crate) async fn get_project(&self, id: &str) -> Result<ProjectResponse, String> {
        let path = format!("project/{id}");
        let request = self.prepare_request(&path, None);
        self.fetch(request).await
    }

    /// The projects of `ids` (ids or slugs) in one request, in no particular order. An id
    /// Modrinth does not know is left out, and so is an entry of a shape not expected.
    pub(crate) async fn get_projects_metadata(
        &self,
        ids: &[String],
    ) -> Result<Vec<ProjectMetadataResponse>, String> {
        let request = self.prepare_request("projects", Some(ids_query(ids)?));
        let entries: Vec<Lenient<ProjectMetadataResponse>> = self.fetch(request).await?;
        Ok(entries.into_iter().filter_map(Lenient::parsed).collect())
    }

    /// The members of the teams of `ids` in one request, one list per team, in no particular
    /// order. A team Modrinth does not know is left out, and so is a member of a shape not
    /// expected.
    pub(crate) async fn get_teams(
        &self,
        ids: &[String],
    ) -> Result<Vec<Vec<TeamMemberResponse>>, String> {
        let request = self.prepare_request_at(&self.v3_base_url, "teams", Some(ids_query(ids)?));
        let teams: Vec<Vec<Lenient<TeamMemberResponse>>> = self.fetch(request).await?;
        Ok(teams
            .into_iter()
            .map(|members| members.into_iter().filter_map(Lenient::parsed).collect())
            .collect())
    }

    /// The organizations of `ids` in one request, in no particular order. An organization
    /// Modrinth does not know is left out, and so is an entry of a shape not expected.
    pub(crate) async fn get_organizations(
        &self,
        ids: &[String],
    ) -> Result<Vec<OrganizationResponse>, String> {
        let request =
            self.prepare_request_at(&self.v3_base_url, "organizations", Some(ids_query(ids)?));
        let entries: Vec<Lenient<OrganizationResponse>> = self.fetch(request).await?;
        Ok(entries.into_iter().filter_map(Lenient::parsed).collect())
    }

    /// The versions of `ids` in one request, in no particular order. An id Modrinth does not
    /// know is left out, and so is an entry of a shape not expected.
    pub(crate) async fn get_versions_metadata(
        &self,
        ids: &[String],
    ) -> Result<Vec<VersionMetadataResponse>, String> {
        let request = self.prepare_request("versions", Some(ids_query(ids)?));
        let entries: Vec<Lenient<VersionMetadataResponse>> = self.fetch(request).await?;
        Ok(entries.into_iter().filter_map(Lenient::parsed).collect())
    }

    pub(crate) async fn get_project_version(
        &self,
        id: &str,
    ) -> Result<ProjectVersionResponse, String> {
        let path = format!("version/{id}");
        let request = self.prepare_request(&path, None);
        self.fetch(request).await
    }

    pub(crate) async fn get_project_versions(
        &self,
        id: &str,
        params: &ProjectVersionsRequest,
    ) -> Result<Vec<ProjectVersionResponse>, String> {
        let query = serde_qs::to_string(params).map_err(|e| e.to_string())?;
        let path = format!("project/{id}/version");
        let request = self.prepare_request(&path, Some(query));
        self.fetch(request).await
    }
}

/// The `ids=[...]` query of the routes that take many ids at once.
fn ids_query(ids: &[String]) -> Result<String, String> {
    let ids = serde_json::to_string(ids).map_err(|e| e.to_string())?;
    Ok(url::form_urlencoded::Serializer::new(String::new())
        .append_pair("ids", &ids)
        .finish())
}
