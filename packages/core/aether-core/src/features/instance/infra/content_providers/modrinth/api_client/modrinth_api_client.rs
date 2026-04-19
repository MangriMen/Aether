use crate::libs::request_client::{Request, RequestClient, RequestClientExt};
use std::sync::Arc;

use super::models::{
    ProjectResponse, ProjectSearchParams, ProjectSearchResponse, ProjectVersionResponse,
    ProjectVersionsRequest,
};

pub struct ModrinthApiClient<RC> {
    base_url: String,
    base_headers: Option<reqwest::header::HeaderMap>,
    request_client: Arc<RC>,
}

impl<RC: RequestClient> ModrinthApiClient<RC> {
    pub fn new(
        base_url: String,
        base_headers: Option<reqwest::header::HeaderMap>,
        request_client: Arc<RC>,
    ) -> Self {
        Self {
            base_url,
            base_headers,
            request_client,
        }
    }

    fn prepare_request(&self, path: &str, query: Option<String>) -> Request {
        let base = self.base_url.trim_end_matches('/');
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

    pub async fn search(
        &self,
        params: &ProjectSearchParams,
    ) -> Result<ProjectSearchResponse, String> {
        let query = serde_qs::to_string(params).map_err(|e| e.to_string())?;
        let request = self.prepare_request("search", Some(query));
        self.fetch(request).await
    }

    pub async fn get_project(&self, id: &str) -> Result<ProjectResponse, String> {
        let path = format!("project/{id}");
        let request = self.prepare_request(&path, None);
        self.fetch(request).await
    }

    pub async fn get_project_version(&self, id: &str) -> Result<ProjectVersionResponse, String> {
        let path = format!("version/{id}");
        let request = self.prepare_request(&path, None);
        self.fetch(request).await
    }

    pub async fn get_project_versions(
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
