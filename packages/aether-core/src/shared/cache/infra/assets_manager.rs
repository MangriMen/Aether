use std::{io::Read, path::Path, sync::Arc};

use async_trait::async_trait;
use bytes::Bytes;

use crate::shared::{
    cache::domain::{AssetError, AssetsStorage, CacheId, CacheKey, FileStore},
    hash::infra::sha1_async,
    image::{looks_like_svg, sniff_image_extension},
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

/// An SVG is a document that can carry scripts and references to other files, and the asset is
/// served to the UI as it is stored, so what can run or fetch is stripped first. A file that
/// does not parse is no image.
async fn sanitize_svg(data: Vec<u8>) -> Result<Vec<u8>, AssetError> {
    tokio::task::spawn_blocking(move || {
        let mut clean = Vec::with_capacity(data.len());
        svg_hush::Filter::new()
            .filter(data.as_slice(), &mut clean)
            .map(|()| clean)
            .map_err(|_| AssetError::NotAnImage)
    })
    .await
    .map_err(|err| AssetError::ReadSource(err.to_string()))?
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
        let (data, ext) = if let Some(ext) = sniff_image_extension(&data) {
            (data, ext)
        } else if looks_like_svg(&data) {
            (sanitize_svg(data).await?, "svg")
        } else {
            return Err(AssetError::NotAnImage);
        };
        let bytes = Bytes::from_owner(data);

        let hash = sha1_async(bytes.clone())
            .await
            .map_err(|err| AssetError::ReadSource(err.to_string()))?;

        let filename = format!("{hash}.{ext}");
        let key = CacheKey::new(ASSETS_CACHE_NAMESPACE, CacheId::Named(filename.clone()));

        if !self.cache.exists(&key).await {
            // The asset is what the instance will point at: a file not written is a failed import.
            self.cache
                .write(&key, bytes)
                .await
                .map_err(|err| AssetError::Write(err.to_string()))?;
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
        async fn write(&self, _: &CacheKey<()>, _: Bytes) -> std::io::Result<()> {
            Ok(())
        }
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

    #[derive(Default)]
    struct WritingStore {
        written: Mutex<Option<Vec<u8>>>,
    }

    #[async_trait]
    impl FileStore for WritingStore {
        async fn exists(&self, _: &CacheKey<()>) -> bool {
            false
        }
        async fn read(&self, _: &CacheKey<()>) -> Option<Bytes> {
            None
        }
        async fn write(&self, _: &CacheKey<()>, bytes: Bytes) -> std::io::Result<()> {
            *self.written.lock().unwrap() = Some(bytes.to_vec());
            Ok(())
        }
        async fn invalidate(&self, _: &CacheKey<()>) {}
        async fn delete(&self, _: &CacheKey<()>) -> std::io::Result<()> {
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
    async fn should_store_an_svg_without_what_runs_in_it() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" onload="alert(1)"><script>alert(2)</script><rect width="1" height="1"/></svg>"#;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("icon.svg");
        std::fs::write(&path, svg).unwrap();
        let store = Arc::new(WritingStore::default());

        let name = FsAssetsStorage::new(store.clone())
            .import_file(&path)
            .await
            .unwrap();

        assert_eq!(Path::new(&name).extension().unwrap(), "svg");
        let stored = String::from_utf8(store.written.lock().unwrap().clone().unwrap()).unwrap();
        assert!(stored.contains("<rect"), "{stored}");
        assert!(
            !stored.contains("script") && !stored.contains("onload"),
            "{stored}"
        );
    }

    #[tokio::test]
    async fn should_refuse_an_svg_that_does_not_parse() {
        let result = import(b"<svg><rect></svg>", "icon.svg").await;

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
