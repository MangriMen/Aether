use async_trait::async_trait;
use serde::de::DeserializeOwned;
use tracing::trace;

use crate::{
    features::events::ProgressBarId,
    libs::request_client::{Request, RequestClient, RequestError},
};

#[async_trait]
pub trait RequestClientExt: RequestClient {
    async fn fetch_json<T>(&self, request: Request) -> Result<T, RequestError>
    where
        T: DeserializeOwned,
    {
        self.fetch_json_with_progress(request, None).await
    }

    #[tracing::instrument(skip(self, loading_bar))]
    async fn fetch_json_with_progress<T>(
        &self,
        request: Request,
        loading_bar: Option<(&ProgressBarId, f64)>,
    ) -> Result<T, RequestError>
    where
        T: DeserializeOwned,
    {
        let bytes = self.fetch_bytes_with_progress(request, loading_bar).await?;

        serde_json::from_slice(&bytes).map_err(|err| {
            trace!("{err:?}");
            if let Ok(text) = std::str::from_utf8(&bytes) {
                trace!("Response body: {text}");
            }

            Into::into(err)
        })
    }

    async fn fetch_toml<T>(&self, request: Request) -> Result<T, RequestError>
    where
        T: DeserializeOwned,
    {
        self.fetch_toml_with_progress(request, None).await
    }

    async fn fetch_toml_with_progress<T>(
        &self,
        request: Request,
        loading_bar: Option<(&ProgressBarId, f64)>,
    ) -> Result<T, RequestError>
    where
        T: DeserializeOwned,
    {
        let response_data = self.fetch_bytes_with_progress(request, loading_bar).await?;
        let toml_str = std::str::from_utf8(&response_data)?;
        toml::from_str(toml_str).map_err(Into::into)
    }
}

#[async_trait]
impl<T: RequestClient + Sync> RequestClientExt for T {}
