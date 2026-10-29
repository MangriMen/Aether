use std::net::{IpAddr, SocketAddr};

use aether_core_plugin_api::v0::FileHashDto;
use async_trait::async_trait;

use crate::features::events::{EventError, ProgressBarId, ProgressEventType};
use crate::test_support::link_dir;

use super::super::test_server::{TestServer, ok, redirect, status};
use super::*;

const BODY: &[u8] = b"hello, cache";

/// Two loopback addresses: two different hosts for `allowed_hosts` and the address filter.
const FIRST: &str = "127.0.0.1";
const SECOND: &str = "127.0.0.2";

struct NoProgress;

#[async_trait]
impl ProgressService for NoProgress {
    async fn init_progress(
        &self,
        _event_type: ProgressEventType,
        _total: f64,
        _message: String,
    ) -> Result<ProgressBarId, EventError> {
        Err(EventError::NotInitialized)
    }
    async fn init_or_edit_progress(
        &self,
        _progress_bar_id: Option<ProgressBarId>,
        _event_type: ProgressEventType,
        _total: f64,
        _message: String,
    ) -> Result<ProgressBarId, EventError> {
        Err(EventError::NotInitialized)
    }
    async fn emit_progress(
        &self,
        _progress_bar_id: &ProgressBarId,
        _increment_frac: f64,
        _message: Option<&str>,
    ) -> Result<(), EventError> {
        Ok(())
    }
    async fn edit_progress(
        &self,
        _progress_bar_id: &ProgressBarId,
        _event_type: ProgressEventType,
        _total: f64,
        _message: String,
    ) -> Result<(), EventError> {
        Ok(())
    }
}

/// The test server lives on loopback, which the production filter blocks.
fn allow_any_address(_ip: IpAddr) -> bool {
    false
}

fn sha1_of(bytes: &[u8]) -> String {
    sha1_smol::Sha1::from(bytes).hexdigest()
}

fn loopback() -> Vec<String> {
    vec!["127.0.0.1".to_string()]
}

fn context<'a>(cache_dir: &'a Path, allowed_hosts: &'a [String]) -> DownloadContext<'a> {
    DownloadContext {
        cache_dir,
        allowed_hosts,
        semaphore: Arc::new(FetchSemaphore(tokio::sync::Semaphore::new(4))),
        progress_service: Arc::new(NoProgress),
        proxy: ProxySource::Direct,
        limits: PluginSandboxLimits::default(),
        budget: DOWNLOAD_BUDGET,
    }
}

fn params(addr: SocketAddr, hash: FileHashDto) -> DownloadToCacheParamsDto {
    DownloadToCacheParamsDto {
        url: format!("http://127.0.0.1:{}/files/mod.jar", addr.port()),
        hash,
    }
}

fn ok_response(body: &[u8]) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

/// Serve exactly one request with the given raw response, then close.
async fn serve_once(raw_response: Vec<u8>) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test server should bind");
    let addr = listener.local_addr().expect("test server should have addr");

    tokio::spawn(async move {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let (mut stream, _) = listener.accept().await.expect("should accept connection");

        // Read the whole request head, so the close below does not reset the connection
        // under a client that is still writing.
        let mut received = Vec::new();
        let mut buf = [0_u8; 1024];
        while !received.windows(4).any(|window| window == b"\r\n\r\n") {
            match stream.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(n) => received.extend_from_slice(&buf[..n]),
            }
        }

        let _ = stream.write_all(&raw_response).await;
        let _ = stream.flush().await;
    });

    addr
}

/// Every entry under `dir`, recursively, as paths relative to it.
fn entries(dir: &Path) -> Vec<PathBuf> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(read) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in read.flatten() {
            let path = entry.path();
            out.push(path.strip_prefix(root).unwrap().to_path_buf());
            if path.is_dir() {
                walk(root, &path, out);
            }
        }
    }

    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

fn only_directories(dir: &Path) -> bool {
    entries(dir).iter().all(|entry| dir.join(entry).is_dir())
}

#[tokio::test]
async fn should_download_verify_and_return_a_cache_path_when_the_hash_matches() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = loopback();
    let hex = sha1_of(BODY);
    let addr = serve_once(ok_response(BODY)).await;

    let file = perform_download(
        params(addr, FileHashDto::Sha1(hex.to_ascii_uppercase())),
        &context(cache.path(), &hosts),
        allow_any_address,
    )
    .await
    .expect("download should succeed");

    assert_eq!(file.path, format!("/cache/downloads/sha1/{hex}"));
    assert_eq!(file.size, BODY.len() as u64);

    let on_disk = cache.path().join("downloads").join("sha1").join(&hex);
    assert_eq!(std::fs::read(&on_disk).unwrap(), BODY);
    assert_eq!(
        entries(cache.path()),
        vec![
            PathBuf::from("downloads"),
            PathBuf::from("downloads").join("sha1"),
            PathBuf::from("downloads").join("sha1").join(&hex),
        ],
        "nothing but the verified file may be left in the cache"
    );
}

/// Answer one request with the headers and a first piece of a body, then hold the connection
/// open without sending the rest: a transfer the client's own timeouts give up on only after
/// `READ_TIMEOUT`.
async fn serve_a_stalled_body() -> SocketAddr {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test server should bind");
    let addr = listener.local_addr().expect("test server should have addr");

    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).await;
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\n\r\nfirst bytes")
            .await
            .expect("write the start of the response");
        std::future::pending::<()>().await;
        drop(stream);
    });

    addr
}

#[tokio::test]
async fn should_give_up_and_leave_nothing_in_the_cache_when_the_budget_runs_out() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = loopback();
    let addr = serve_a_stalled_body().await;
    let budget = Duration::from_millis(300);
    let context = DownloadContext {
        budget,
        ..context(cache.path(), &hosts)
    };

    let err = perform_download(
        params(addr, FileHashDto::Sha1(sha1_of(BODY))),
        &context,
        allow_any_address,
    )
    .await
    .expect_err("a download past its budget must fail");

    assert!(
        matches!(err, PluginDownloadError::OutOfTime(cut) if cut == budget),
        "cut off by the budget, long before the client's own timeouts: {err}"
    );
    assert!(
        only_directories(cache.path()),
        "the partial download must be removed: {:?}",
        entries(cache.path())
    );
}

#[tokio::test]
async fn should_fail_and_leave_nothing_in_the_cache_when_the_hash_does_not_match() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = loopback();
    let wrong = sha1_of(b"something else");
    let addr = serve_once(ok_response(BODY)).await;

    let err = perform_download(
        params(addr, FileHashDto::Sha1(wrong.clone())),
        &context(cache.path(), &hosts),
        allow_any_address,
    )
    .await
    .expect_err("a body that does not match the hash must be refused");

    assert!(
        matches!(
            err,
            PluginDownloadError::Request(RequestError::Hash { ref expected, .. })
                if *expected == wrong
        ),
        "expected a hash mismatch, got: {err}"
    );
    assert!(
        !cache
            .path()
            .join("downloads")
            .join("sha1")
            .join(&wrong)
            .exists(),
        "the rejected file must not appear under its final name"
    );
    assert!(
        only_directories(cache.path()),
        "no partial file may be left behind: {:?}",
        entries(cache.path())
    );
}

#[tokio::test]
async fn should_verify_sha256_and_sha512_when_the_plugin_gives_either() {
    for (hash, algorithm) in [
        (
            FileHashDto::Sha256(
                crate::shared::hash::infra::hash_hex_async(HashAlgorithm::Sha256, BODY)
                    .await
                    .unwrap(),
            ),
            "sha256",
        ),
        (
            FileHashDto::Sha512(
                crate::shared::hash::infra::hash_hex_async(HashAlgorithm::Sha512, BODY)
                    .await
                    .unwrap(),
            ),
            "sha512",
        ),
    ] {
        let cache = tempfile::tempdir().unwrap();
        let hosts = loopback();
        let addr = serve_once(ok_response(BODY)).await;

        let file = perform_download(
            params(addr, hash),
            &context(cache.path(), &hosts),
            allow_any_address,
        )
        .await
        .unwrap_or_else(|err| panic!("{algorithm} download should succeed: {err}"));

        assert!(
            file.path
                .starts_with(&format!("/cache/downloads/{algorithm}/")),
            "{}",
            file.path
        );
    }
}

#[tokio::test]
async fn should_reuse_a_cached_file_without_touching_the_network_when_it_is_valid() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = loopback();
    let hex = sha1_of(BODY);
    let addr = serve_once(ok_response(BODY)).await;

    perform_download(
        params(addr, FileHashDto::Sha1(hex.clone())),
        &context(cache.path(), &hosts),
        allow_any_address,
    )
    .await
    .expect("first download should succeed");

    // The server answered once and is gone: only a cache hit can succeed now.
    let file = perform_download(
        params(addr, FileHashDto::Sha1(hex.clone())),
        &context(cache.path(), &hosts),
        allow_any_address,
    )
    .await
    .expect("second call should be served from the cache");

    assert_eq!(file.path, format!("/cache/downloads/sha1/{hex}"));
    assert_eq!(file.size, BODY.len() as u64);
}

#[tokio::test]
async fn should_replace_a_planted_file_when_it_does_not_match_its_name() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = loopback();
    let hex = sha1_of(BODY);

    // `/cache` is writable from the sandbox: the plugin may have put anything there.
    let planted = cache.path().join("downloads").join("sha1").join(&hex);
    std::fs::create_dir_all(planted.parent().unwrap()).unwrap();
    std::fs::write(&planted, b"not the real thing").unwrap();

    let addr = serve_once(ok_response(BODY)).await;
    perform_download(
        params(addr, FileHashDto::Sha1(hex)),
        &context(cache.path(), &hosts),
        allow_any_address,
    )
    .await
    .expect("a stale entry must be replaced by a fresh download");

    assert_eq!(std::fs::read(&planted).unwrap(), BODY);
}

// ── Redirects ──

fn download_params(url: String) -> DownloadToCacheParamsDto {
    DownloadToCacheParamsDto {
        url,
        hash: FileHashDto::Sha1(sha1_of(BODY)),
    }
}

#[tokio::test]
async fn should_hash_only_the_final_body_when_a_redirect_leads_to_an_allowlisted_host() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = vec![FIRST.to_string(), SECOND.to_string()];
    let target = TestServer::start(SECOND, |_| ok(BODY)).await;
    let location = target.url("/release-assets/mod.jar");
    // The redirect carries a body of its own; it must not end up in the file or the hash.
    let origin = TestServer::start(FIRST, move |_| redirect(302, &location)).await;

    let file = perform_download(
        download_params(origin.url("/releases/latest/download/mod.jar")),
        &context(cache.path(), &hosts),
        allow_any_address,
    )
    .await
    .expect("an allowlisted hop should be followed");

    let hex = sha1_of(BODY);
    assert_eq!(file.path, format!("/cache/downloads/sha1/{hex}"));
    assert_eq!(file.size, BODY.len() as u64);
    let on_disk = cache.path().join("downloads").join("sha1").join(&hex);
    assert_eq!(std::fs::read(on_disk).unwrap(), BODY);
}

#[tokio::test]
async fn should_refuse_a_redirect_without_connecting_when_it_leads_outside_allowed_hosts() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = loopback();
    let target = TestServer::start(SECOND, |_| ok(BODY)).await;
    let location = target.url("/mod.jar");
    let origin = TestServer::start(FIRST, move |_| redirect(302, &location)).await;

    let err = perform_download(
        download_params(origin.url("/mod.jar")),
        &context(cache.path(), &hosts),
        allow_any_address,
    )
    .await
    .expect_err("a hop to a host outside allowed_hosts must be refused");

    assert!(
        matches!(
            err,
            PluginDownloadError::Http(PluginHttpError::HostNotAllowed { ref host })
                if host == SECOND
        ),
        "expected HostNotAllowed, got: {err}"
    );
    assert_eq!(
        target.connections(),
        0,
        "the refused host must not be contacted"
    );
    assert_eq!(origin.connections(), 1, "a policy refusal is not retried");
    assert!(
        only_directories(cache.path()),
        "{:?}",
        entries(cache.path())
    );
}

#[tokio::test]
async fn should_fail_without_retrying_when_the_chain_has_a_sixth_redirect() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = loopback();
    let server = TestServer::start(FIRST, |path| {
        let n: usize = path
            .strip_prefix("/hops/")
            .and_then(|n| n.parse().ok())
            .expect("chain path");
        if n == 0 {
            ok(BODY)
        } else {
            redirect(302, &format!("/hops/{}", n - 1))
        }
    })
    .await;

    perform_download(
        download_params(server.url("/hops/5")),
        &context(cache.path(), &hosts),
        allow_any_address,
    )
    .await
    .expect("five redirects are within the limit");

    let cache = tempfile::tempdir().unwrap();
    let before = server.connections();
    let err = perform_download(
        download_params(server.url("/hops/6")),
        &context(cache.path(), &hosts),
        allow_any_address,
    )
    .await
    .expect_err("a sixth redirect must be an error");

    assert!(
        matches!(
            err,
            PluginDownloadError::Http(PluginHttpError::TooManyRedirects { limit: 5 })
        ),
        "expected TooManyRedirects, got: {err}"
    );
    assert_eq!(
        server.connections() - before,
        6,
        "six responses, the sixth Location never requested, no retry"
    );
    assert!(
        only_directories(cache.path()),
        "{:?}",
        entries(cache.path())
    );
}

#[tokio::test]
async fn should_retry_from_the_start_of_the_chain_when_a_failure_is_transient() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = vec![FIRST.to_string(), SECOND.to_string()];
    let attempts = std::sync::atomic::AtomicUsize::new(0);
    let target = TestServer::start(SECOND, move |_| {
        if attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
            status(503)
        } else {
            ok(BODY)
        }
    })
    .await;
    let location = target.url("/mod.jar");
    let origin = TestServer::start(FIRST, move |_| redirect(302, &location)).await;

    perform_download(
        download_params(origin.url("/mod.jar")),
        &context(cache.path(), &hosts),
        allow_any_address,
    )
    .await
    .expect("the retry should succeed");

    // The signed URL a CDN hands out may expire: a retry asks the origin again.
    assert_eq!(origin.connections(), 2);
    assert_eq!(target.connections(), 2);
}

#[tokio::test]
async fn should_hash_only_the_final_body_when_downloading_through_a_proxy() {
    // Behind a proxy, names go to the proxy unresolved (`.invalid` never resolves)
    // while `allowed_hosts` still applies to every hop.
    let cache = tempfile::tempdir().unwrap();
    let hosts = vec![
        "mods.plugin.invalid".to_string(),
        "cdn.plugin.invalid".to_string(),
    ];
    let proxy = TestServer::start(FIRST, |target| match target {
        "http://mods.plugin.invalid/mod.jar" => {
            redirect(302, "http://cdn.plugin.invalid/signed/mod.jar")
        }
        _ => ok(BODY),
    })
    .await;
    let mut context = context(cache.path(), &hosts);
    context.proxy = ProxySource::Fixed(proxy.url(""));

    let file = perform_download(
        download_params("http://mods.plugin.invalid/mod.jar".to_string()),
        &context,
        is_blocked_ip,
    )
    .await
    .expect("a proxied download should succeed");

    let on_disk = cache
        .path()
        .join("downloads")
        .join("sha1")
        .join(sha1_of(BODY));
    assert_eq!(file.size, BODY.len() as u64);
    assert_eq!(std::fs::read(on_disk).unwrap(), BODY);
    assert_eq!(proxy.requests().len(), 2);
}

#[tokio::test]
async fn should_fail_when_the_status_is_an_error() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = loopback();
    let addr =
        serve_once(b"HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nnot found".to_vec()).await;

    let err = perform_download(
        params(addr, FileHashDto::Sha1(sha1_of(b"not found"))),
        &context(cache.path(), &hosts),
        allow_any_address,
    )
    .await
    .expect_err("a 404 body must not be cached, even if its hash happens to match");

    assert!(
        matches!(
            err,
            PluginDownloadError::Request(RequestError::Status { status: 404, .. })
        ),
        "expected a 404 status error, got: {err}"
    );
    assert!(
        only_directories(cache.path()),
        "{:?}",
        entries(cache.path())
    );
}

#[tokio::test]
async fn should_refuse_a_body_before_reading_it_when_it_is_over_the_limit() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = loopback();
    let addr = serve_once(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\nstart",
            PluginSandboxLimits::default().max_download_bytes + 1
        )
        .into_bytes(),
    )
    .await;

    let err = perform_download(
        params(addr, FileHashDto::Sha1(sha1_of(BODY))),
        &context(cache.path(), &hosts),
        allow_any_address,
    )
    .await
    .expect_err("an oversized body must be refused");

    assert!(
        matches!(
            err,
            PluginDownloadError::Request(RequestError::TooLarge { .. })
        ),
        "expected TooLarge, got: {err}"
    );
    assert!(
        only_directories(cache.path()),
        "{:?}",
        entries(cache.path())
    );
}

#[tokio::test]
async fn should_reject_a_host_before_touching_the_cache_when_it_is_outside_allowed_hosts() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = vec!["example.com".to_string()];

    let err = handle_download_to_cache(
        &PluginId::from_static("test"),
        DownloadToCacheParamsDto {
            url: "http://evil.com/mod.jar".to_string(),
            hash: FileHashDto::Sha1(sha1_of(BODY)),
        },
        &context(cache.path(), &hosts),
    )
    .await
    .expect_err("a host outside allowed_hosts must be refused");

    assert!(err.to_string().contains("allowed_hosts"), "{err}");
    assert!(entries(cache.path()).is_empty());
}

#[tokio::test]
async fn should_block_non_public_addresses_when_the_production_filter_is_used() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = vec!["*".to_string()];

    for url in [
        "http://127.0.0.1:9/mod.jar",
        "http://169.254.169.254/latest/meta-data/",
    ] {
        let err = handle_download_to_cache(
            &PluginId::from_static("test"),
            DownloadToCacheParamsDto {
                url: url.to_string(),
                hash: FileHashDto::Sha1(sha1_of(BODY)),
            },
            &context(cache.path(), &hosts),
        )
        .await
        .expect_err("a non-public address must be refused");

        assert!(
            err.to_string().contains("non-public address"),
            "{url}: {err}"
        );
    }
}

#[tokio::test]
async fn should_reject_a_hash_before_any_io_when_it_is_not_a_digest() {
    let cache = tempfile::tempdir().unwrap();
    let hosts = vec!["*".to_string()];
    let not_hex = "g".repeat(40);

    for hex in [
        "../../../../../../../../../../../../../evil",
        "abc",
        not_hex.as_str(),
    ] {
        let err = handle_download_to_cache(
            &PluginId::from_static("test"),
            DownloadToCacheParamsDto {
                url: "https://example.com/mod.jar".to_string(),
                hash: FileHashDto::Sha1(hex.to_string()),
            },
            &context(cache.path(), &hosts),
        )
        .await
        .expect_err("a malformed digest must be refused");

        assert!(err.to_string().contains("Invalid hash"), "{hex}: {err}");
    }
    assert!(entries(cache.path()).is_empty());
}

#[tokio::test]
async fn should_refuse_a_downloads_directory_when_it_links_out_of_the_cache() {
    let root = tempfile::tempdir().unwrap();
    let cache = root.path().join("cache");
    let outside = root.path().join("outside");
    std::fs::create_dir_all(&cache).unwrap();
    std::fs::create_dir_all(&outside).unwrap();

    let link = cache.join("downloads");
    link_dir(&outside, &link);

    let hosts = loopback();
    let addr = serve_once(ok_response(BODY)).await;
    let err = perform_download(
        params(addr, FileHashDto::Sha1(sha1_of(BODY))),
        &context(&cache, &hosts),
        allow_any_address,
    )
    .await
    .expect_err("a link out of the cache must not be written through");

    assert!(
        matches!(err, PluginDownloadError::UnusableCache { .. }),
        "expected UnusableCache, got: {err}"
    );
    assert!(
        entries(&outside).is_empty(),
        "nothing may be created outside the cache: {:?}",
        entries(&outside)
    );
}
