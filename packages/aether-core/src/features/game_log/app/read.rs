use std::sync::Arc;

use async_trait::async_trait;

use crate::features::{
    game_log::domain::{GameLogError, LogCursor, LogPage, LogSource, LogSourceRef},
    instance::InstanceId,
};

/// A page is never smaller than this, so a caller cannot make the reader crawl a file by lines.
const MIN_PAGE_BYTES: u64 = 4 * 1024;
const MAX_PAGE_BYTES: u64 = 1024 * 1024;

/// Where the log files of an instance are kept.
#[async_trait]
pub trait GameLogStore: Send + Sync {
    /// Console logs first (newest first), then the game's own logs.
    async fn list(&self, instance_id: &InstanceId) -> Result<Vec<LogSource>, GameLogError>;

    async fn read(
        &self,
        instance_id: &InstanceId,
        source: &LogSourceRef,
        cursor: LogCursor,
        max_bytes: u64,
    ) -> Result<LogPage, GameLogError>;
}

#[async_trait]
pub trait ListGameLogSourcesUseCasePort: Send + Sync {
    async fn execute(&self, instance_id: InstanceId) -> Result<Vec<LogSource>, GameLogError>;
}

#[async_trait]
pub trait ReadGameLogUseCasePort: Send + Sync {
    async fn execute(
        &self,
        instance_id: InstanceId,
        source_id: String,
        cursor: LogCursor,
        max_bytes: u64,
    ) -> Result<LogPage, GameLogError>;
}

pub struct ListGameLogSourcesUseCase {
    store: Arc<dyn GameLogStore>,
}

impl ListGameLogSourcesUseCase {
    pub fn new(store: Arc<dyn GameLogStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl ListGameLogSourcesUseCasePort for ListGameLogSourcesUseCase {
    async fn execute(&self, instance_id: InstanceId) -> Result<Vec<LogSource>, GameLogError> {
        self.store.list(&instance_id).await
    }
}

pub struct ReadGameLogUseCase {
    store: Arc<dyn GameLogStore>,
}

impl ReadGameLogUseCase {
    pub fn new(store: Arc<dyn GameLogStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl ReadGameLogUseCasePort for ReadGameLogUseCase {
    async fn execute(
        &self,
        instance_id: InstanceId,
        source_id: String,
        cursor: LogCursor,
        max_bytes: u64,
    ) -> Result<LogPage, GameLogError> {
        let source = LogSourceRef::parse(&source_id)?;
        let max_bytes = max_bytes.clamp(MIN_PAGE_BYTES, MAX_PAGE_BYTES);
        self.store
            .read(&instance_id, &source, cursor, max_bytes)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    /// Records the page size of the last read.
    #[derive(Default)]
    struct FakeStore {
        last_max_bytes: Mutex<Option<u64>>,
    }

    #[async_trait]
    impl GameLogStore for FakeStore {
        async fn list(&self, _: &InstanceId) -> Result<Vec<LogSource>, GameLogError> {
            Ok(Vec::new())
        }

        async fn read(
            &self,
            _: &InstanceId,
            _: &LogSourceRef,
            _: LogCursor,
            max_bytes: u64,
        ) -> Result<LogPage, GameLogError> {
            *self.last_max_bytes.lock().unwrap() = Some(max_bytes);
            Ok(LogPage {
                text: String::new(),
                start: 0,
                end: 0,
                has_more_before: false,
                has_more_after: false,
            })
        }
    }

    fn id() -> InstanceId {
        InstanceId::try_from("inst".to_string()).unwrap()
    }

    #[tokio::test]
    async fn should_reject_the_read_when_the_source_id_is_not_a_log() {
        let store = Arc::new(FakeStore::default());
        let use_case = ReadGameLogUseCase::new(store.clone());

        let err = use_case
            .execute(id(), "game:../x.log".into(), LogCursor::Tail, 10_000)
            .await
            .unwrap_err();

        assert!(matches!(err, GameLogError::InvalidSource { .. }));
        assert_eq!(*store.last_max_bytes.lock().unwrap(), None);
    }

    #[tokio::test]
    async fn should_clamp_the_page_size_when_the_caller_asks_for_too_much() {
        let store = Arc::new(FakeStore::default());
        let use_case = ReadGameLogUseCase::new(store.clone());

        use_case
            .execute(id(), "game:latest.log".into(), LogCursor::Tail, u64::MAX)
            .await
            .unwrap();

        assert_eq!(*store.last_max_bytes.lock().unwrap(), Some(MAX_PAGE_BYTES));
    }
}
