use std::path::PathBuf;
use std::sync::Arc;

use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, trace};

use crate::shared::{
    io::{ensure_read_json_async, write_json_async},
    read_json_async, IoError, UpdateAction,
};
pub struct JsonValueStore<T> {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> JsonValueStore<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            lock: Arc::new(Mutex::new(())),
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn read(&self) -> Result<T, IoError> {
        let _guard = self.lock.lock().await;
        read_json_async(&self.path).await
    }

    pub async fn read_or_default(&self) -> Result<T, IoError>
    where
        T: Default,
    {
        let _guard = self.lock.lock().await;
        ensure_read_json_async(&self.path).await
    }

    pub async fn write(&self, data: &T) -> Result<(), IoError> {
        let _guard = self.lock.lock().await;
        write_json_async(&self.path, data).await
    }

    /// Atomic update: Read -> Modify -> Write.
    /// To prevent Race Conditions
    pub async fn update<F, R>(&self, f: F) -> Result<R, IoError>
    where
        F: FnOnce(&mut T) -> UpdateAction<R>,
    {
        let _guard = self.lock.lock().await;
        let data: T = read_json_async(&self.path).await?;

        self.apply_and_save(data, f).await
    }

    pub async fn update_with_default<F, R>(&self, f: F) -> Result<R, IoError>
    where
        T: Default,
        F: FnOnce(&mut T) -> UpdateAction<R>,
    {
        let _guard = self.lock.lock().await;
        let data: T = ensure_read_json_async(&self.path).await?;

        self.apply_and_save(data, f).await
    }

    /// Mutex must be locked
    async fn apply_and_save<F, R>(&self, mut data: T, f: F) -> Result<R, IoError>
    where
        F: FnOnce(&mut T) -> UpdateAction<R>,
    {
        match f(&mut data) {
            UpdateAction::Save(result) => {
                write_json_async(&self.path, &data).await?;
                debug!(path = ?self.path, "Store updated and saved to disk");
                Ok(result)
            }
            UpdateAction::NoChanges(result) => {
                trace!(path = ?self.path, "Update called but no changes detected");
                Ok(result)
            }
        }
    }
}
