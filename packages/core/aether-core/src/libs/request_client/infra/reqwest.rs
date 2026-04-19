use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Response;

use crate::{
    features::events::{ProgressBarId, ProgressService, ProgressServiceExt},
    libs::request_client::{Method, Request, RequestClient, RequestError},
    shared::{FetchSemaphore, sha1_async},
};

pub struct ReqwestClient<PS: ProgressService> {
    progress_service: Arc<PS>,
    client: Arc<reqwest_middleware::ClientWithMiddleware>,
    semaphore: Arc<FetchSemaphore>,
}

impl<PS: ProgressService> ReqwestClient<PS> {
    pub fn new(
        progress_service: Arc<PS>,
        client: Arc<reqwest_middleware::ClientWithMiddleware>,
        semaphore: Arc<FetchSemaphore>,
    ) -> Self {
        Self {
            progress_service,
            client,
            semaphore,
        }
    }

    async fn fetch_chunks(
        &self,
        response: Response,
        progress_bar: (&ProgressBarId, f64),
    ) -> Result<Bytes, RequestError> {
        use futures::StreamExt;

        let Some(total_size) = response.content_length() else {
            return Ok(response.bytes().await?);
        };

        let mut stream = response.bytes_stream();
        let mut bytes = Vec::new();

        let (progress_bar_id, total) = progress_bar;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(RequestError::RequestSendError)?;
            bytes.extend_from_slice(&chunk);

            #[allow(clippy::cast_precision_loss)]
            let progress = (chunk.len() as f64 / total_size as f64) * total;

            self.progress_service
                .emit_progress_safe(progress_bar_id, progress, None)
                .await;
        }

        Ok(bytes.into())
    }

    async fn verify_sha1(&self, bytes: Bytes, expected_sha1: String) -> Result<(), RequestError> {
        let actual_sha1 = sha1_async(bytes).await;

        if actual_sha1 == *expected_sha1 {
            Ok(())
        } else {
            Err(RequestError::HashError {
                actual: actual_sha1,
                expected: expected_sha1,
            })
        }
    }
}

#[async_trait]
impl<PS: ProgressService> RequestClient for ReqwestClient<PS> {
    async fn fetch_bytes(&self, request: Request) -> Result<Bytes, RequestError> {
        self.fetch_bytes_with_progress(request, None).await
    }

    async fn fetch_bytes_with_progress(
        &self,
        request: Request,
        progress_bar: Option<(&ProgressBarId, f64)>,
    ) -> Result<Bytes, RequestError> {
        let Request {
            method,
            url,
            headers,
            sha1,
            body,
        } = request;

        let _permit = self.semaphore.0.acquire().await?;

        let mut request = self.client.request(method.into(), &url);

        if let Some(body) = body {
            request = request.body(body);
        }

        if let Some(headers) = headers {
            request = request.headers(headers);
        }

        let response = request.send().await.map_err(|e| match e {
            reqwest_middleware::Error::Middleware(error) => RequestError::MiddlewareError(error),
            reqwest_middleware::Error::Reqwest(error) => RequestError::RequestSendError(error),
        })?;

        let bytes = match progress_bar {
            Some(progress_bar) => self.fetch_chunks(response, progress_bar).await,
            None => response.bytes().await.map_err(Into::into),
        }?;

        if let Some(expected_sha1) = sha1 {
            self.verify_sha1(bytes.clone(), expected_sha1).await?;
        }

        Ok(bytes)
    }
}

impl From<Method> for reqwest::Method {
    fn from(method: Method) -> Self {
        match method {
            Method::Options => reqwest::Method::OPTIONS,
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Head => reqwest::Method::HEAD,
            Method::Trace => reqwest::Method::TRACE,
            Method::Connect => reqwest::Method::CONNECT,
            Method::Patch => reqwest::Method::PATCH,
        }
    }
}
