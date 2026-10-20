use tokio::sync::Semaphore;

#[derive(Debug)]
pub struct FetchSemaphore(pub Semaphore);

/// Limits the light requests (metadata, search), kept apart from [`FetchSemaphore`] so that a
/// long queue of file downloads never holds them back.
#[derive(Debug)]
pub struct ApiSemaphore(pub Semaphore);

impl ApiSemaphore {
    pub const DEFAULT_PERMITS: usize = 16;
}

impl Default for ApiSemaphore {
    fn default() -> Self {
        Self(Semaphore::new(Self::DEFAULT_PERMITS))
    }
}
