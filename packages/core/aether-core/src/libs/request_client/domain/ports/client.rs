use async_trait::async_trait;
use bytes::Bytes;

use crate::{
    features::events::ProgressBarId,
    libs::request_client::{Request, RequestError},
};

#[async_trait]
pub trait RequestClient: Send + Sync {
    async fn fetch_bytes(&self, request: Request) -> Result<Bytes, RequestError>;

    async fn fetch_bytes_with_progress(
        &self,
        request: Request,
        loading_bar: Option<(&ProgressBarId, f64)>,
    ) -> Result<Bytes, RequestError>;
}
