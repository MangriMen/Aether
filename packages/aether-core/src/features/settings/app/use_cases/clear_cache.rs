use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::settings::app::ports::ClearCacheUseCasePort,
    shared::housekeeping::{CacheCleaner, clear_caches},
};

/// Empties the caches the user may clear: what the app can fetch again.
pub struct ClearCacheUseCase {
    cleaners: Vec<Arc<dyn CacheCleaner>>,
}

impl ClearCacheUseCase {
    pub fn new(cleaners: Vec<Arc<dyn CacheCleaner>>) -> Self {
        Self { cleaners }
    }
}

#[async_trait]
impl ClearCacheUseCasePort for ClearCacheUseCase {
    async fn execute(&self) {
        clear_caches(&self.cleaners).await;
    }
}
