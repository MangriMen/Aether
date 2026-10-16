//! [`ContentIconCache`] as files in the asset store, which the UI already reads its local images
//! from.
//!
//! An icon is named by a hash of its URL, never by anything in the URL, so no URL can name a
//! path; the extension is that of the image format found in the body. A URL that names its
//! content by hash (`…/<sha1>_96.webp`, most of Modrinth's) never changes and is never asked
//! about again. One that does not (`…/icon.png`) is asked once it is older than
//! [`CONTENT_METADATA_TTL`], with the `ETag` and `Last-Modified` kept beside it in
//! `<name>.json`.
//!
//! Cleanup goes by the URLs still wanted (`retain`), not by time; a file's mtime is only when it
//! was last written, which spares a fresh icon from a cleanup that cannot know its owner yet.

use std::{
    collections::HashSet,
    fmt::Write as _,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    features::instance::app::{
        CONTENT_METADATA_TTL, CachedContentIcon, ContentIconCache, ContentIconError,
    },
    shared::{
        housekeeping::{CacheCleaner, CacheKind},
        request_client::{FileDownload, Request, RequestClient},
    },
};

/// The folder of the asset store the icons are kept in.
pub const CONTENT_ICONS_FOLDER: &str = "content-icons";

/// The largest icon taken. Modrinth's are a few KiB.
const MAX_ICON_SIZE: u64 = 1 << 20;

/// How long fetching one icon may take.
const FETCH_TIMEOUT: Duration = Duration::from_secs(20);

/// How old an icon must be to be dropped for being unwanted: a younger one may belong to
/// content whose metadata the cleanup did not see yet.
const FRESH_AGE: Duration = Duration::from_hours(1);

/// How old a partial file must be to be taken for the leftover of an interrupted fetch rather
/// than one in progress.
const LEFTOVER_AGE: Duration = Duration::from_hours(1);

/// The formats taken, as the extensions their files get. Most likely first: a lookup tries
/// them in turn.
const EXTENSIONS: [&str; 4] = ["webp", "png", "jpg", "gif"];

const PARTIAL_PREFIXES: [&str; 2] = [".icon-", ".download-"];

/// What to ask the server with whether an icon changed, kept as `<name>.json`.
#[derive(Serialize, Deserialize, Default)]
struct Validators {
    etag: Option<String>,
    last_modified: Option<String>,
    /// When the server last gave or confirmed the icon, in seconds since the Unix epoch.
    checked_at: u64,
}

pub struct FsContentIconCache {
    client: Arc<dyn RequestClient>,
    dir: PathBuf,
}

impl FsContentIconCache {
    /// Icons under [`CONTENT_ICONS_FOLDER`] of the asset store at `assets_dir`.
    pub fn new(client: Arc<dyn RequestClient>, assets_dir: &Path) -> Self {
        Self {
            client,
            dir: assets_dir.join(CONTENT_ICONS_FOLDER),
        }
    }

    /// The icon of `stem`, if any: its path and extension.
    async fn find(&self, stem: &str) -> Option<(PathBuf, &'static str, std::fs::Metadata)> {
        for extension in EXTENSIONS {
            let path = self.dir.join(format!("{stem}.{extension}"));
            if let Ok(metadata) = tokio::fs::metadata(&path).await
                && metadata.is_file()
            {
                return Some((path, extension, metadata));
            }
        }
        None
    }

    fn validators_path(&self, stem: &str) -> PathBuf {
        self.dir.join(format!("{stem}.json"))
    }

    async fn read_validators(&self, stem: &str) -> Option<Validators> {
        let bytes = tokio::fs::read(self.validators_path(stem)).await.ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    async fn write_validators(&self, stem: &str, validators: &Validators) {
        let Ok(bytes) = serde_json::to_vec(validators) else {
            return;
        };
        if let Err(err) = write_atomic(&self.dir, &self.validators_path(stem), bytes).await {
            tracing::debug!("Failed to keep the validators of a content icon: {err}");
        }
    }
}

/// The name of the icon of `url`, without extension.
fn stem(url: &str) -> String {
    let digest = Sha256::digest(url.as_bytes());
    digest[..16]
        .iter()
        .fold(String::with_capacity(32), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

/// Whether `url` names its content by hash, so the same URL never brings another icon: a run
/// of 32 or more hex digits in its path, as the sha1 of Modrinth's `…/<sha1>_96.webp`.
fn names_content_by_hash(url: &url::Url) -> bool {
    url.path()
        .split(|c: char| !c.is_ascii_hexdigit())
        .any(|run| run.len() >= 32)
}

fn parse(url: &str) -> Result<url::Url, ContentIconError> {
    let parsed = url::Url::parse(url).map_err(|_| ContentIconError::InvalidUrl(url.to_owned()))?;
    if matches!(parsed.scheme(), "https" | "http") {
        Ok(parsed)
    } else {
        Err(ContentIconError::InvalidUrl(url.to_owned()))
    }
}

/// The extension of the image format `head` (the first bytes of a file) starts, if taken.
fn sniff_extension(head: &[u8]) -> Option<&'static str> {
    if head.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else if head.starts_with(b"\xff\xd8\xff") {
        Some("jpg")
    } else if head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a") {
        Some("gif")
    } else if head.len() >= 12 && &head[..4] == b"RIFF" && &head[8..12] == b"WEBP" {
        Some("webp")
    } else {
        None
    }
}

/// Whether a `Content-Type` says image. A missing one leaves it to the body.
fn is_image_type(content_type: Option<&str>) -> bool {
    content_type.is_none_or(|value| {
        value
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .starts_with("image/")
    })
}

async fn read_head(path: &Path) -> std::io::Result<Vec<u8>> {
    let path = path.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut head = Vec::with_capacity(16);
        std::fs::File::open(path)?.take(16).read_to_end(&mut head)?;
        Ok(head)
    })
    .await
    .map_err(std::io::Error::other)?
}

/// Write `bytes` to `path` all at once: through a partial file in `dir`, renamed over it.
async fn write_atomic(dir: &Path, path: &Path, bytes: Vec<u8>) -> std::io::Result<()> {
    let dir = dir.to_owned();
    let path = path.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut file = tempfile::Builder::new()
            .prefix(PARTIAL_PREFIXES[0])
            .suffix(".part")
            .tempfile_in(&dir)?;
        std::io::Write::write_all(&mut file, &bytes)?;
        file.persist(&path).map_err(|err| err.error)?;
        Ok(())
    })
    .await
    .map_err(std::io::Error::other)?
}

fn unix_secs(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

fn storage_error(err: impl std::fmt::Display) -> ContentIconError {
    ContentIconError::Storage(err.to_string())
}

#[async_trait]
impl ContentIconCache for FsContentIconCache {
    async fn lookup(&self, url: &str) -> CachedContentIcon {
        let Ok(parsed) = parse(url) else {
            return CachedContentIcon::Missing;
        };
        let stem = stem(url);
        let Some((_, extension, _)) = self.find(&stem).await else {
            return CachedContentIcon::Missing;
        };

        let now = SystemTime::now();

        // A URL that names its content by hash never brings another file. Any other one does
        // only on a fetch that writes the validators anew, so their time tells the files apart.
        let (revalidate, revision) = if names_content_by_hash(&parsed) {
            (false, None)
        } else {
            let checked_at = self
                .read_validators(&stem)
                .await
                .map(|validators| validators.checked_at);
            let revalidate = checked_at.is_none_or(|checked_at| {
                unix_secs(now).saturating_sub(checked_at) > CONTENT_METADATA_TTL.as_secs()
            });
            (revalidate, Some(checked_at.unwrap_or_default()))
        };

        CachedContentIcon::Present {
            asset_id: format!("{CONTENT_ICONS_FOLDER}/{stem}.{extension}"),
            revalidate,
            revision,
        }
    }

    async fn fetch(&self, url: &str) -> Result<bool, ContentIconError> {
        let parsed = parse(url)?;
        let immutable = names_content_by_hash(&parsed);
        let stem = stem(url);

        let existing = self.find(&stem).await;
        if existing.is_some() && immutable {
            return Ok(false);
        }

        let mut request = Request::get(url).with_timeout(FETCH_TIMEOUT);
        if existing.is_some()
            && let Some(validators) = self.read_validators(&stem).await
        {
            let mut headers = reqwest::header::HeaderMap::new();
            let pairs = [
                (reqwest::header::IF_NONE_MATCH, validators.etag),
                (reqwest::header::IF_MODIFIED_SINCE, validators.last_modified),
            ];
            for (name, value) in pairs {
                if let Some(value) = value.and_then(|value| value.parse().ok()) {
                    headers.insert(name, value);
                }
            }
            request = request.with_headers(headers);
        }

        tokio::fs::create_dir_all(&self.dir)
            .await
            .map_err(storage_error)?;
        // The body lands here first, and goes under its final name only once it is known to be
        // an image. Deleted when dropped, on any early return.
        let staging = tempfile::Builder::new()
            .prefix(PARTIAL_PREFIXES[0])
            .suffix(".part")
            .tempfile_in(&self.dir)
            .map_err(storage_error)?
            .into_temp_path();

        let saved = match self
            .client
            .download_to_file_conditional(request, &staging, MAX_ICON_SIZE)
            .await
            .map_err(|err| ContentIconError::Request(err.to_string()))?
        {
            FileDownload::NotModified if existing.is_some() => {
                let validators = self.read_validators(&stem).await.unwrap_or_default();
                self.write_validators(
                    &stem,
                    &Validators {
                        checked_at: unix_secs(SystemTime::now()),
                        ..validators
                    },
                )
                .await;
                return Ok(false);
            }
            FileDownload::NotModified => {
                return Err(ContentIconError::Request(
                    "not modified, with nothing cached".into(),
                ));
            }
            FileDownload::Saved(saved) => saved,
        };

        if !is_image_type(saved.content_type.as_deref()) {
            return Err(ContentIconError::NotAnImage);
        }
        let head = read_head(&staging).await.map_err(storage_error)?;
        let Some(extension) = sniff_extension(&head) else {
            return Err(ContentIconError::NotAnImage);
        };

        staging
            .persist(self.dir.join(format!("{stem}.{extension}")))
            .map_err(|err| storage_error(err.error))?;

        if let Some((old_path, old_extension, _)) = existing
            && old_extension != extension
            && let Err(err) = tokio::fs::remove_file(&old_path).await
        {
            tracing::debug!("Failed to remove a replaced content icon: {err}");
        }

        if !immutable {
            self.write_validators(
                &stem,
                &Validators {
                    etag: saved.etag,
                    last_modified: saved.last_modified,
                    checked_at: unix_secs(SystemTime::now()),
                },
            )
            .await;
        }

        Ok(true)
    }

    async fn retain(&self, keep: &HashSet<String>) -> usize {
        let Ok(mut entries) = tokio::fs::read_dir(&self.dir).await else {
            return 0;
        };

        let now = SystemTime::now();
        let kept: HashSet<String> = keep.iter().map(|url| stem(url)).collect();
        let mut removed = 0;
        let mut validators = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Ok(metadata) = entry.metadata().await else {
                continue;
            };
            if !metadata.is_file() {
                continue;
            }
            let age = metadata
                .modified()
                .ok()
                .and_then(|modified| now.duration_since(modified).ok())
                .unwrap_or_default();

            let extension = path.extension().and_then(|extension| extension.to_str());
            let is_partial = PARTIAL_PREFIXES
                .iter()
                .any(|prefix| name.starts_with(prefix));
            let is_icon = !is_partial && extension.is_some_and(|ext| EXTENSIONS.contains(&ext));

            if extension == Some("json") && !is_partial {
                validators.push(path);
                continue;
            }
            let unwanted = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|stem| !kept.contains(stem));
            let expired =
                (is_partial && age >= LEFTOVER_AGE) || (is_icon && unwanted && age >= FRESH_AGE);
            if !expired {
                continue;
            }
            match tokio::fs::remove_file(&path).await {
                Ok(()) if is_icon => removed += 1,
                Ok(()) => {}
                Err(err) => tracing::debug!("Failed to remove {}: {err}", path.display()),
            }
        }

        // The validators of an icon that is gone, removed just now or before.
        for path in validators {
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if self.find(stem).await.is_none()
                && let Err(err) = tokio::fs::remove_file(&path).await
            {
                tracing::debug!("Failed to remove {}: {err}", path.display());
            }
        }

        removed
    }
}

/// Every icon and its validators, and the leftovers of fetches. The lists fetch again what they
/// show.
#[async_trait]
impl CacheCleaner for FsContentIconCache {
    fn what(&self) -> &'static str {
        "content icons"
    }

    fn kind(&self) -> CacheKind {
        CacheKind::Icons
    }

    async fn size(&self) -> u64 {
        crate::shared::housekeeping::files_size(&self.dir).await
    }

    async fn clear(&self) -> u64 {
        let Ok(mut entries) = tokio::fs::read_dir(&self.dir).await else {
            return 0;
        };

        let mut removed = 0;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if !entry
                .metadata()
                .await
                .is_ok_and(|metadata| metadata.is_file())
            {
                continue;
            }
            // A partial file may be a fetch in progress, which removes its own on failure.
            let is_partial = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    PARTIAL_PREFIXES
                        .iter()
                        .any(|prefix| name.starts_with(prefix))
                });
            if is_partial {
                continue;
            }
            let is_icon = path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| EXTENSIONS.contains(&extension));
            match tokio::fs::remove_file(&path).await {
                Ok(()) if is_icon => removed += 1,
                Ok(()) => {}
                Err(err) => tracing::debug!("Failed to remove {}: {err}", path.display()),
            }
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use bytes::Bytes;

    use super::*;
    use crate::{
        features::events::ProgressBarId,
        shared::request_client::{BytesProgress, RequestError, SavedDownload},
    };

    const HASHED_URL: &str =
        "https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp";
    const LEGACY_URL: &str = "https://cdn.modrinth.com/data/gvQqBUqZ/icon.png";

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n-png-body";
    const WEBP: &[u8] = b"RIFF\x10\x00\x00\x00WEBPVP8 -webp-body";

    #[derive(Clone)]
    enum Reply {
        Image {
            body: &'static [u8],
            content_type: Option<&'static str>,
            etag: Option<&'static str>,
        },
        Fail,
    }

    fn image(body: &'static [u8], content_type: &'static str) -> Reply {
        Reply::Image {
            body,
            content_type: Some(content_type),
            etag: None,
        }
    }

    /// Answers by URL; an `If-None-Match` matching the reply's `ETag` gets a 304.
    #[derive(Default)]
    struct FakeHttp {
        replies: Mutex<HashMap<String, Reply>>,
        /// Each request: its URL and `If-None-Match`.
        requests: Mutex<Vec<(String, Option<String>)>>,
    }

    impl FakeHttp {
        fn reply(&self, url: &str, reply: Reply) {
            self.replies.lock().unwrap().insert(url.into(), reply);
        }

        fn requests(&self) -> Vec<(String, Option<String>)> {
            self.requests.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl RequestClient for FakeHttp {
        async fn fetch_bytes(&self, _: Request) -> Result<Bytes, RequestError> {
            unimplemented!()
        }

        async fn fetch_bytes_with_progress(
            &self,
            _: Request,
            _: Option<(&ProgressBarId, f64)>,
        ) -> Result<Bytes, RequestError> {
            unimplemented!()
        }

        async fn download_to_file(
            &self,
            _: Request,
            _: &Path,
            _: u64,
            _: Option<BytesProgress<'_>>,
        ) -> Result<u64, RequestError> {
            unimplemented!()
        }

        async fn download_to_file_conditional(
            &self,
            request: Request,
            destination: &Path,
            max_size: u64,
        ) -> Result<FileDownload, RequestError> {
            let if_none_match = request
                .headers
                .as_ref()
                .and_then(|headers| headers.get(reqwest::header::IF_NONE_MATCH))
                .map(|value| value.to_str().unwrap().to_owned());
            self.requests
                .lock()
                .unwrap()
                .push((request.url.clone(), if_none_match.clone()));

            let reply = self.replies.lock().unwrap().get(&request.url).cloned();
            match reply {
                Some(Reply::Image {
                    body,
                    content_type,
                    etag,
                }) => {
                    if etag.is_some() && if_none_match.as_deref() == etag {
                        return Ok(FileDownload::NotModified);
                    }
                    if body.len() as u64 > max_size {
                        return Err(RequestError::TooLarge { limit: max_size });
                    }
                    std::fs::write(destination, body).unwrap();
                    Ok(FileDownload::Saved(SavedDownload {
                        size: body.len() as u64,
                        content_type: content_type.map(Into::into),
                        etag: etag.map(Into::into),
                        last_modified: None,
                    }))
                }
                Some(Reply::Fail) | None => Err(RequestError::Status {
                    status: 503,
                    url: request.url,
                }),
            }
        }
    }

    struct Fixture {
        _temp: tempfile::TempDir,
        dir: PathBuf,
        http: Arc<FakeHttp>,
        cache: FsContentIconCache,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().unwrap();
            let http = Arc::new(FakeHttp::default());
            let cache = FsContentIconCache::new(http.clone(), temp.path());
            Self {
                dir: temp.path().join(CONTENT_ICONS_FOLDER),
                _temp: temp,
                http,
                cache,
            }
        }

        fn files(&self) -> Vec<String> {
            let Ok(entries) = std::fs::read_dir(&self.dir) else {
                return Vec::new();
            };
            let mut names: Vec<String> = entries
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        }

        fn path_of(&self, asset_id: &str) -> PathBuf {
            self.dir
                .parent()
                .unwrap()
                .join(asset_id.replace('/', std::path::MAIN_SEPARATOR_STR))
        }

        async fn present(&self, url: &str) -> (String, bool) {
            match self.cache.lookup(url).await {
                CachedContentIcon::Present {
                    asset_id,
                    revalidate,
                    ..
                } => (asset_id, revalidate),
                CachedContentIcon::Missing => panic!("no icon for {url}"),
            }
        }

        async fn revision(&self, url: &str) -> Option<u64> {
            match self.cache.lookup(url).await {
                CachedContentIcon::Present { revision, .. } => revision,
                CachedContentIcon::Missing => panic!("no icon for {url}"),
            }
        }

        /// Make the validators of `url` look checked `age` ago.
        fn age_validators(&self, url: &str, age: Duration) {
            let path = self.dir.join(format!("{}.json", stem(url)));
            let mut validators: Validators =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            validators.checked_at = unix_secs(SystemTime::now() - age);
            std::fs::write(path, serde_json::to_vec(&validators).unwrap()).unwrap();
        }
    }

    fn set_age(path: &Path, age: Duration) {
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(SystemTime::now() - age)
            .unwrap();
    }

    #[test]
    fn should_take_a_url_as_naming_its_content_when_its_path_has_a_long_hex_run() {
        assert!(names_content_by_hash(&url::Url::parse(HASHED_URL).unwrap()));
        assert!(!names_content_by_hash(
            &url::Url::parse(LEGACY_URL).unwrap()
        ));
        assert!(!names_content_by_hash(
            &url::Url::parse("https://cdn.example.com/deadbeef/icon.png").unwrap()
        ));
    }

    #[tokio::test]
    async fn should_name_the_icon_by_url_hash_and_body_format_when_it_is_fetched() {
        let fixture = Fixture::new();
        // Claimed PNG, really WEBP: the body decides.
        fixture.http.reply(HASHED_URL, image(WEBP, "image/png"));

        assert_eq!(
            fixture.cache.lookup(HASHED_URL).await,
            CachedContentIcon::Missing
        );
        assert!(fixture.cache.fetch(HASHED_URL).await.unwrap());

        let (asset_id, revalidate) = fixture.present(HASHED_URL).await;
        let name = format!("{}.webp", stem(HASHED_URL));
        assert_eq!(asset_id, format!("{CONTENT_ICONS_FOLDER}/{name}"));
        assert!(!revalidate);
        assert_eq!(fixture.revision(HASHED_URL).await, None);
        assert_eq!(std::fs::read(fixture.path_of(&asset_id)).unwrap(), WEBP);
        // Nothing but the icon: no partial file, no validators for an immutable URL.
        assert_eq!(fixture.files(), [name]);
    }

    #[tokio::test]
    async fn should_not_ask_again_for_an_icon_when_its_url_names_it_by_hash() {
        let fixture = Fixture::new();
        fixture.http.reply(HASHED_URL, image(WEBP, "image/webp"));
        fixture.cache.fetch(HASHED_URL).await.unwrap();

        assert!(!fixture.cache.fetch(HASHED_URL).await.unwrap());

        assert_eq!(fixture.http.requests().len(), 1);
    }

    #[tokio::test]
    async fn should_reject_a_response_when_it_is_not_an_image() {
        let fixture = Fixture::new();
        let html = "https://cdn.example.com/a/icon.png";
        let garbage = "https://cdn.example.com/b/icon.png";
        let svg = "https://cdn.example.com/c/icon.svg";
        fixture
            .http
            .reply(html, image(PNG, "text/html; charset=utf-8"));
        fixture.http.reply(garbage, image(b"<html>", "image/png"));
        fixture.http.reply(
            svg,
            image(
                b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
                "image/svg+xml",
            ),
        );

        for url in [html, garbage, svg] {
            assert!(
                matches!(
                    fixture.cache.fetch(url).await,
                    Err(ContentIconError::NotAnImage)
                ),
                "{url}"
            );
            assert_eq!(fixture.cache.lookup(url).await, CachedContentIcon::Missing);
        }
        assert!(fixture.files().is_empty(), "{:?}", fixture.files());
    }

    #[tokio::test]
    async fn should_reject_an_icon_when_it_is_too_large() {
        let fixture = Fixture::new();
        let huge: &'static [u8] =
            Box::leak(vec![0_u8; usize::try_from(MAX_ICON_SIZE).unwrap() + 1].into());
        fixture.http.reply(LEGACY_URL, image(huge, "image/png"));

        assert!(matches!(
            fixture.cache.fetch(LEGACY_URL).await,
            Err(ContentIconError::Request(_))
        ));
        assert!(fixture.files().is_empty(), "{:?}", fixture.files());
    }

    #[tokio::test]
    async fn should_reject_a_url_when_it_is_not_http() {
        let fixture = Fixture::new();

        for url in ["file:///etc/passwd", "not a url", "../../icon.png"] {
            assert!(matches!(
                fixture.cache.fetch(url).await,
                Err(ContentIconError::InvalidUrl(_))
            ));
            assert_eq!(fixture.cache.lookup(url).await, CachedContentIcon::Missing);
        }
        assert!(fixture.http.requests().is_empty());
    }

    #[tokio::test]
    async fn should_keep_the_icon_when_a_due_revalidation_is_answered_not_modified() {
        let fixture = Fixture::new();
        fixture.http.reply(
            LEGACY_URL,
            Reply::Image {
                body: PNG,
                content_type: Some("image/png"),
                etag: Some("\"v1\""),
            },
        );
        fixture.cache.fetch(LEGACY_URL).await.unwrap();
        assert!(!fixture.present(LEGACY_URL).await.1, "fresh");

        fixture.age_validators(LEGACY_URL, CONTENT_METADATA_TTL + Duration::from_hours(1));
        assert!(fixture.present(LEGACY_URL).await.1, "due");

        assert!(!fixture.cache.fetch(LEGACY_URL).await.unwrap());

        let requests = fixture.http.requests();
        assert_eq!(requests[1], (LEGACY_URL.into(), Some("\"v1\"".into())));
        let (asset_id, revalidate) = fixture.present(LEGACY_URL).await;
        assert!(!revalidate, "confirmed just now");
        assert_eq!(std::fs::read(fixture.path_of(&asset_id)).unwrap(), PNG);
    }

    #[tokio::test]
    async fn should_replace_the_icon_when_a_revalidation_brings_another() {
        let fixture = Fixture::new();
        fixture.http.reply(LEGACY_URL, image(PNG, "image/png"));
        fixture.cache.fetch(LEGACY_URL).await.unwrap();
        fixture.http.reply(LEGACY_URL, image(WEBP, "image/webp"));

        assert!(fixture.cache.fetch(LEGACY_URL).await.unwrap());

        let (asset_id, _) = fixture.present(LEGACY_URL).await;
        assert_eq!(
            Path::new(&asset_id)
                .extension()
                .and_then(|ext| ext.to_str()),
            Some("webp")
        );
        let stem = stem(LEGACY_URL);
        assert_eq!(
            fixture.files(),
            [format!("{stem}.json"), format!("{stem}.webp")]
        );
    }

    #[tokio::test]
    async fn should_change_the_revision_when_a_revalidation_brings_another_icon() {
        let fixture = Fixture::new();
        fixture.http.reply(LEGACY_URL, image(PNG, "image/png"));
        fixture.cache.fetch(LEGACY_URL).await.unwrap();
        fixture.age_validators(LEGACY_URL, CONTENT_METADATA_TTL + Duration::from_hours(1));
        let before = fixture.revision(LEGACY_URL).await;
        fixture.http.reply(LEGACY_URL, image(PNG, "image/png"));

        assert!(fixture.cache.fetch(LEGACY_URL).await.unwrap());

        let after = fixture.revision(LEGACY_URL).await;
        assert!(before.is_some() && after.is_some());
        assert_ne!(before, after);
    }

    #[tokio::test]
    async fn should_keep_the_old_icon_when_a_revalidation_fails() {
        let fixture = Fixture::new();
        fixture.http.reply(LEGACY_URL, image(PNG, "image/png"));
        fixture.cache.fetch(LEGACY_URL).await.unwrap();
        fixture.http.reply(LEGACY_URL, Reply::Fail);

        assert!(fixture.cache.fetch(LEGACY_URL).await.is_err());
        fixture
            .http
            .reply(LEGACY_URL, image(b"<html>", "text/html"));
        assert!(fixture.cache.fetch(LEGACY_URL).await.is_err());

        let (asset_id, _) = fixture.present(LEGACY_URL).await;
        assert_eq!(std::fs::read(fixture.path_of(&asset_id)).unwrap(), PNG);
        assert_eq!(fixture.files().len(), 2, "{:?}", fixture.files());
    }

    #[tokio::test]
    async fn should_keep_wanted_and_fresh_icons_and_remove_the_rest_when_retaining() {
        let fixture = Fixture::new();
        let wanted = "https://cdn.example.com/wanted/icon.png";
        let fresh = "https://cdn.example.com/fresh/icon.png";
        for url in [wanted, fresh, LEGACY_URL] {
            fixture.http.reply(url, image(PNG, "image/png"));
            fixture.cache.fetch(url).await.unwrap();
        }
        let (old_icon, _) = fixture.present(wanted).await;
        set_age(
            &fixture.path_of(&old_icon),
            FRESH_AGE + Duration::from_secs(60),
        );
        let (unwanted_icon, _) = fixture.present(LEGACY_URL).await;
        set_age(
            &fixture.path_of(&unwanted_icon),
            FRESH_AGE + Duration::from_secs(60),
        );
        let leftover = fixture.dir.join(".icon-abc.part");
        std::fs::write(&leftover, b"partial").unwrap();
        set_age(&leftover, LEFTOVER_AGE + Duration::from_secs(60));
        let in_progress = fixture.dir.join(".download-def.part");
        std::fs::write(&in_progress, b"partial").unwrap();

        let keep = HashSet::from([wanted.to_owned()]);
        assert_eq!(fixture.cache.retain(&keep).await, 1);

        let (wanted_stem, fresh_stem) = (stem(wanted), stem(fresh));
        assert_eq!(
            fixture.files(),
            [
                ".download-def.part".to_owned(),
                format!("{fresh_stem}.json"),
                format!("{fresh_stem}.png"),
                format!("{wanted_stem}.json"),
                format!("{wanted_stem}.png"),
            ]
        );
    }

    #[tokio::test]
    async fn should_remove_every_icon_when_cleared() {
        let fixture = Fixture::new();
        fixture.http.reply(HASHED_URL, image(WEBP, "image/webp"));
        fixture.http.reply(LEGACY_URL, image(PNG, "image/png"));
        fixture.cache.fetch(HASHED_URL).await.unwrap();
        fixture.cache.fetch(LEGACY_URL).await.unwrap();
        let in_progress = fixture.dir.join(".download-def.part");
        std::fs::write(&in_progress, b"partial").unwrap();

        assert_eq!(fixture.cache.clear().await, 2);

        assert_eq!(fixture.files(), [".download-def.part".to_owned()]);
        assert_eq!(
            fixture.cache.lookup(HASHED_URL).await,
            CachedContentIcon::Missing
        );
    }
}
