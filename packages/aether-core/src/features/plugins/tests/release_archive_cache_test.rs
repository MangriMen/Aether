use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use async_trait::async_trait;

use crate::features::plugins::{
    PluginError, PluginProvider, PluginProviderError, PluginSourceType, ProviderPluginPreview,
    ProviderUpdateInfo, ReleaseArchiveCache, download_to_temp_file,
};

/// A provider that counts its downloads, and can be told to fail.
#[derive(Default)]
struct Counting {
    downloads: AtomicUsize,
    /// The next download fails, and the ones after it do not.
    fails: std::sync::atomic::AtomicBool,
    /// Every URL asked for, in order.
    urls: std::sync::Mutex<Vec<String>>,
    /// What a download of a `slow` URL waits for.
    gate: tokio::sync::Notify,
}

#[async_trait]
impl PluginProvider for Counting {
    async fn fetch_preview(&self, _: &str) -> Result<ProviderPluginPreview, PluginProviderError> {
        unimplemented!()
    }

    async fn fetch_latest_version(
        &self,
        _: &str,
        _: &str,
        _: &str,
    ) -> Result<ProviderUpdateInfo, PluginProviderError> {
        unimplemented!()
    }

    async fn download_plugin(
        &self,
        url: &str,
        dest: &mut tokio::fs::File,
    ) -> Result<u64, PluginProviderError> {
        use tokio::io::AsyncWriteExt;

        self.downloads.fetch_add(1, Ordering::SeqCst);
        self.urls.lock().unwrap().push(url.to_owned());
        if url.starts_with("hang") {
            std::future::pending::<()>().await;
        }
        if url.starts_with("slow") {
            self.gate.notified().await;
        }
        if self.fails.swap(false, Ordering::SeqCst) {
            return Err(PluginProviderError::ProviderFetchError {
                source_type: PluginSourceType::GitHub,
                details: "connection dropped".into(),
            });
        }
        dest.write_all(url.as_bytes()).await.unwrap();
        Ok(url.len() as u64)
    }

    fn source_type(&self) -> PluginSourceType {
        PluginSourceType::GitHub
    }

    fn display_name(&self) -> String {
        "Stub source".to_owned()
    }

    fn parse_identifier(&self, raw: &str) -> Result<String, PluginProviderError> {
        Ok(raw.to_owned())
    }
}

/// A cache whose clock only moves when told to, and the handle that moves it.
fn cache_with_clock() -> (Arc<ReleaseArchiveCache>, Arc<AtomicU64>) {
    let start = Instant::now();
    let elapsed_secs = Arc::new(AtomicU64::new(0));
    let clock = {
        let elapsed_secs = elapsed_secs.clone();
        move || start + Duration::from_secs(elapsed_secs.load(Ordering::SeqCst))
    };
    (
        Arc::new(ReleaseArchiveCache::with_clock(Box::new(clock))),
        elapsed_secs,
    )
}

async fn get(
    cache: &Arc<ReleaseArchiveCache>,
    provider: &Counting,
    url: &str,
) -> Result<Arc<crate::features::plugins::DownloadedArchive>, PluginError> {
    cache
        .get_or_download(url, || download_to_temp_file(provider, url))
        .await
}

#[tokio::test]
async fn should_download_again_once_the_archive_is_older_than_the_ttl() {
    let (cache, clock) = cache_with_clock();
    let provider = Counting::default();

    get(&cache, &provider, "u").await.unwrap();
    clock.store(10 * 60 - 1, Ordering::SeqCst);
    get(&cache, &provider, "u").await.unwrap();
    assert_eq!(provider.downloads.load(Ordering::SeqCst), 1);

    clock.store(10 * 60, Ordering::SeqCst);
    get(&cache, &provider, "u").await.unwrap();
    assert_eq!(provider.downloads.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn should_let_the_oldest_archive_go_when_a_fifth_is_added() {
    let (cache, clock) = cache_with_clock();
    let provider = Counting::default();

    for (i, url) in ["a", "b", "c", "d", "e"].into_iter().enumerate() {
        clock.store(i as u64, Ordering::SeqCst);
        get(&cache, &provider, url).await.unwrap();
    }
    assert_eq!(provider.downloads.load(Ordering::SeqCst), 5);

    // The four newest are still there, the oldest is not.
    for url in ["b", "c", "d", "e"] {
        get(&cache, &provider, url).await.unwrap();
    }
    assert_eq!(provider.downloads.load(Ordering::SeqCst), 5);
    get(&cache, &provider, "a").await.unwrap();
    assert_eq!(provider.downloads.load(Ordering::SeqCst), 6);
}

#[tokio::test]
async fn should_keep_the_file_of_an_archive_held_while_the_cache_lets_it_go() {
    let (cache, clock) = cache_with_clock();
    let provider = Counting::default();

    let held = get(&cache, &provider, "a").await.unwrap();
    for (i, url) in ["b", "c", "d", "e"].into_iter().enumerate() {
        clock.store(i as u64 + 1, Ordering::SeqCst);
        get(&cache, &provider, url).await.unwrap();
    }

    assert!(held.path().exists());
    let path = held.path().to_owned();
    drop(held);
    assert!(!path.exists());
}

#[tokio::test]
async fn should_download_once_when_the_same_archive_is_asked_for_at_the_same_time() {
    let cache = Arc::new(ReleaseArchiveCache::new());
    let provider = Counting::default();

    let (first, second) = tokio::join!(get(&cache, &provider, "u"), get(&cache, &provider, "u"));

    let (first, second) = (first.unwrap(), second.unwrap());
    assert_eq!(provider.downloads.load(Ordering::SeqCst), 1);
    assert!(Arc::ptr_eq(&first, &second));
}

#[tokio::test]
async fn should_not_keep_a_download_that_failed() {
    let cache = Arc::new(ReleaseArchiveCache::new());
    let provider = Counting::default();

    provider.fails.store(true, Ordering::SeqCst);
    assert!(get(&cache, &provider, "u").await.is_err());

    provider.fails.store(false, Ordering::SeqCst);
    get(&cache, &provider, "u").await.unwrap();
    assert_eq!(provider.downloads.load(Ordering::SeqCst), 2);
}

/// Starts a download of `url` and gives it up once it is under way.
async fn start_and_cancel(cache: &Arc<ReleaseArchiveCache>, provider: &Counting, url: &str) {
    tokio::select! {
        _ = get(cache, provider, url) => unreachable!("the download is not let to finish"),
        () = async {
            while !provider.urls.lock().unwrap().iter().any(|u| u == url) {
                tokio::task::yield_now().await;
            }
        } => {}
    }
}

#[tokio::test]
async fn should_not_keep_a_slot_for_a_download_that_was_given_up_nor_let_it_push_out_a_live_one() {
    let (cache, clock) = cache_with_clock();
    let provider = Arc::new(Counting::default());

    let slow = tokio::spawn({
        let (cache, provider) = (cache.clone(), provider.clone());
        async move { get(&cache, &provider, "slow").await.unwrap() }
    });
    while provider.downloads.load(Ordering::SeqCst) == 0 {
        tokio::task::yield_now().await;
    }
    for (i, url) in ["hang1", "hang2", "hang3", "hang4", "hang5"]
        .into_iter()
        .enumerate()
    {
        clock.store(i as u64, Ordering::SeqCst);
        start_and_cancel(&cache, &provider, url).await;
    }

    provider.gate.notify_waiters();
    let slow = slow.await.unwrap();

    // The download that was under way was not pushed out, so it is the one that is kept.
    let again = get(&cache, &provider, "slow").await.unwrap();
    assert!(Arc::ptr_eq(&slow, &again));
    let urls = provider.urls.lock().unwrap().clone();
    assert_eq!(urls.iter().filter(|u| *u == "slow").count(), 1);
}

#[tokio::test]
async fn should_keep_the_result_of_a_waiting_request_when_the_download_before_it_fails() {
    let cache = Arc::new(ReleaseArchiveCache::new());
    let provider = Counting::default();
    provider.fails.store(true, Ordering::SeqCst);

    let (first, second) = tokio::join!(get(&cache, &provider, "u"), get(&cache, &provider, "u"));
    assert!(first.is_err());
    let second = second.unwrap();

    let third = get(&cache, &provider, "u").await.unwrap();
    assert!(Arc::ptr_eq(&second, &third));
    // The one that failed, and the one that went through.
    assert_eq!(provider.downloads.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn should_drop_an_expired_archive_when_swept_without_anyone_asking_for_it() {
    let (cache, clock) = cache_with_clock();
    let provider = Counting::default();
    let path = get(&cache, &provider, "u").await.unwrap().path().to_owned();

    clock.store(10 * 60 - 1, Ordering::SeqCst);
    cache.sweep();
    assert!(path.exists());

    clock.store(10 * 60, Ordering::SeqCst);
    cache.sweep();
    assert!(!path.exists());
}
