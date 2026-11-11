use std::{io::Read, path::Path, sync::Arc};

use async_trait::async_trait;
use bytes::Bytes;

use crate::shared::{
    cache::domain::{AssetError, AssetsStorage, CacheId, CacheKey, FileStore},
    hash::infra::sha1_async,
    image::sniff_image_extension,
};

pub const ASSETS_CACHE_NAMESPACE: &str = "assets";

/// The largest file `import_file` takes. The source may be a file a plugin wrote anywhere it can
/// write, so it is not read whole before its size is known.
pub const MAX_ASSET_BYTES: u64 = 4 * 1024 * 1024;

async fn read_limited(source: &Path, limit: u64) -> Result<Vec<u8>, AssetError> {
    let source = source.to_owned();
    tokio::task::spawn_blocking(move || {
        // Looked at before it is opened: opening a pipe with no writer would block for good.
        if !std::fs::metadata(&source)?.is_file() {
            return Ok(Err(AssetError::NotAnImage));
        }
        let file = std::fs::File::open(&source)?;
        if file.metadata()?.len() > limit {
            return Ok(Err(AssetError::TooLarge { limit }));
        }
        // The length can grow after the check, so the read is bounded too.
        let mut data = Vec::new();
        file.take(limit + 1).read_to_end(&mut data)?;
        Ok(if data.len() as u64 > limit {
            Err(AssetError::TooLarge { limit })
        } else {
            Ok(data)
        })
    })
    .await
    .map_err(|err| AssetError::ReadSource(err.to_string()))?
    .map_err(|err: std::io::Error| AssetError::ReadSource(err.to_string()))?
}

pub struct FsAssetsStorage<C> {
    cache: Arc<C>,
}

impl<C: FileStore> FsAssetsStorage<C> {
    pub fn new(cache: Arc<C>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl<C: FileStore> AssetsStorage for FsAssetsStorage<C> {
    async fn import_file(&self, source: &Path) -> Result<String, AssetError> {
        let data = read_limited(source, MAX_ASSET_BYTES).await?;
        let ext = sniff_image_extension(&data).ok_or(AssetError::NotAnImage)?;
        let bytes = Bytes::from_owner(data);

        let hash = sha1_async(bytes.clone())
            .await
            .map_err(|err| AssetError::ReadSource(err.to_string()))?;

        let filename = format!("{hash}.{ext}");
        let key = CacheKey::new(ASSETS_CACHE_NAMESPACE, CacheId::Named(filename.clone()));

        if !self.cache.exists(&key).await {
            self.cache.write(&key, bytes).await;
        }

        Ok(filename)
    }

    async fn remove(&self, id: &str) -> Result<(), AssetError> {
        // Ids come from the database: refuse anything that could name a file elsewhere.
        if id.is_empty() || id.contains(['/', '\\']) || id.contains("..") {
            return Ok(());
        }
        let key = CacheKey::new(ASSETS_CACHE_NAMESPACE, CacheId::Named(id.to_owned()));
        self.cache
            .delete(&key)
            .await
            .map_err(|err| AssetError::Remove(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct RecordingStore {
        invalidated: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl FileStore for RecordingStore {
        async fn exists(&self, _: &CacheKey<()>) -> bool {
            false
        }
        async fn read(&self, _: &CacheKey<()>) -> Option<Bytes> {
            None
        }
        async fn write(&self, _: &CacheKey<()>, _: Bytes) {}
        async fn invalidate(&self, key: &CacheKey<()>) {
            if let CacheId::Named(name) = key.id() {
                self.invalidated.lock().unwrap().push(name.clone());
            }
        }
        async fn delete(&self, key: &CacheKey<()>) -> std::io::Result<()> {
            self.invalidate(key).await;
            Ok(())
        }
    }

    const PNG_HEAD: &[u8] = &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

    async fn import(bytes: &[u8], name: &str) -> Result<String, AssetError> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        FsAssetsStorage::new(Arc::new(RecordingStore::default()))
            .import_file(&path)
            .await
    }

    #[tokio::test]
    async fn should_name_the_asset_by_its_format_not_by_the_source_extension() {
        let png = PNG_HEAD;

        let name = import(png, "icon.html").await.unwrap();

        assert_eq!(Path::new(&name).extension().unwrap(), "png");
    }

    #[tokio::test]
    async fn should_refuse_a_file_that_is_not_an_image() {
        let result = import(b"<html><script>", "icon.png").await;

        assert!(matches!(result, Err(AssetError::NotAnImage)));
    }

    #[tokio::test]
    async fn should_refuse_a_directory() {
        let dir = tempfile::tempdir().unwrap();

        let result = FsAssetsStorage::new(Arc::new(RecordingStore::default()))
            .import_file(dir.path())
            .await;

        assert!(matches!(result, Err(AssetError::NotAnImage)));
    }

    #[tokio::test]
    async fn should_refuse_a_file_over_the_limit() {
        let mut data = PNG_HEAD.to_vec();
        data.resize(usize::try_from(MAX_ASSET_BYTES).unwrap() + 1, 0);

        let result = import(&data, "icon.png").await;

        assert!(matches!(result, Err(AssetError::TooLarge { .. })));
    }

    #[tokio::test]
    async fn should_take_a_file_of_exactly_the_limit() {
        let mut data = PNG_HEAD.to_vec();
        data.resize(usize::try_from(MAX_ASSET_BYTES).unwrap(), 0);

        assert!(import(&data, "icon.png").await.is_ok());
    }

    #[tokio::test]
    async fn should_remove_only_plain_asset_names() {
        let store = Arc::new(RecordingStore::default());
        let assets = FsAssetsStorage::new(store.clone());

        for id in ["", "../x.png", "a/b.png", "a\\b.png", "..", "ok.png"] {
            assets.remove(id).await.unwrap();
        }

        assert_eq!(*store.invalidated.lock().unwrap(), ["ok.png"]);
    }
}
