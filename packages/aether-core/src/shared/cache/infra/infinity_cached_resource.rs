use std::{sync::Arc, time::Duration};

use bytes::Bytes;
use tracing::debug;

use crate::shared::{
    cache::domain::{Cache, CacheKey, FileStore},
    io::domain::IoError,
};

pub struct InfinityCachedResource<C> {
    pub cache: Arc<C>,
}

impl<C: Cache> InfinityCachedResource<C> {
    pub fn new(cache: Arc<C>) -> Self {
        Self { cache }
    }

    pub async fn get_cached<T, Fut, KeyFn, ContextFn, E>(
        &self,
        key_fn: KeyFn,
        fetch_fn: Fut,
        context_fn: ContextFn,
        force: bool,
    ) -> Result<T, E>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<T, E>>,
        KeyFn: Fn() -> CacheKey<T>,
        ContextFn: Fn() -> String,
    {
        let key = key_fn();

        if !force && let Some(value) = self.cache.get::<T>(&key).await {
            debug!("Get {} from cache", context_fn());
            return Ok(value);
        }

        let value = fetch_fn.await?;
        debug!("Fetched {}", context_fn());

        self.cache.set(&key, &value, Duration::ZERO).await;

        Ok(value)
    }
}

impl<FS: FileStore> InfinityCachedResource<FS> {
    pub async fn ensure<Fut, KeyFn, ContextFn, E>(
        &self,
        key_fn: KeyFn,
        fetch_fn: Fut,
        context_fn: ContextFn,
        force: bool,
    ) -> Result<(), E>
    where
        E: From<IoError>,
        Fut: std::future::Future<Output = Result<Bytes, E>>,
        KeyFn: Fn() -> CacheKey<()>,
        ContextFn: Fn() -> String,
    {
        let key = key_fn();

        if !force && self.cache.exists(&key).await {
            debug!("{} already downloaded", context_fn());
            return Ok(());
        }

        let value = fetch_fn.await?;
        debug!("{} fetched", context_fn());

        self.cache.write(&key, value).await.map_err(IoError::from)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::shared::cache::{
        domain::{CacheId, CachePathResolver},
        infra::FileCache,
    };

    use super::*;

    struct UnderFile(PathBuf);

    impl CachePathResolver for UnderFile {
        fn resolve(&self, _: &'static str, id: &CacheId) -> Option<PathBuf> {
            Some(self.0.join(id.as_ref()))
        }
    }

    #[tokio::test]
    async fn should_fail_when_the_file_cannot_be_written() {
        let temp = tempfile::tempdir().unwrap();
        // A parent that is a regular file: nothing can be created below it.
        let blocker = temp.path().join("blocker");
        std::fs::write(&blocker, b"x").unwrap();
        let resource = InfinityCachedResource {
            cache: Arc::new(FileCache::new(UnderFile(blocker))),
        };

        let result: Result<(), IoError> = resource
            .ensure(
                || CacheKey::new("test", CacheId::Named("client.jar".to_owned())),
                async { Ok(Bytes::from_static(b"jar")) },
                || "Client".to_owned(),
                false,
            )
            .await;

        assert!(result.is_err(), "{result:?}");
    }
}
