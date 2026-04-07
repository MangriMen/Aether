use std::path::PathBuf;
use std::sync::Arc;

use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, trace};

use crate::shared::{
    io::{ensure_read_json_async, write_json_async},
    IoError,
};

pub enum UpdateAction<R> {
    Save(R),
    NoChanges(R),
}

pub struct JsonEntityStore<T> {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> JsonEntityStore<T>
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

    pub async fn read_all(&self) -> Result<Vec<T>, IoError> {
        let _guard = self.lock.lock().await;
        ensure_read_json_async(&self.path).await
    }

    pub async fn write_all(&self, data: &[T]) -> Result<(), IoError> {
        let _guard = self.lock.lock().await;
        write_json_async(&self.path, data).await
    }

    /// Atomic update: Read -> Modify -> Write.
    /// To prevent Race Conditions
    pub async fn update<F, R>(&self, f: F) -> Result<R, IoError>
    where
        F: FnOnce(&mut Vec<T>) -> UpdateAction<R>,
    {
        let _guard = self.lock.lock().await;

        let mut data: Vec<T> = ensure_read_json_async(&self.path).await?;

        match f(&mut data) {
            UpdateAction::Save(result) => {
                write_json_async(&self.path, data).await?;

                debug!(
                    path = ?self.path,
                    "Entity store updated and saved to disk"
                );

                Ok(result)
            }
            UpdateAction::NoChanges(result) => {
                trace!(
                    path = ?self.path,
                    "Update called but no changes detected; skipping disk IO"
                );

                Ok(result)
            }
        }
    }
}
