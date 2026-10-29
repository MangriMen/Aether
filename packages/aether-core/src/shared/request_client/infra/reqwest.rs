use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Response;
use tokio::io::AsyncWriteExt;

use crate::{
    features::events::{ProgressBarId, ProgressService, ProgressServiceExt},
    shared::{
        fetch::domain::{ApiSemaphore, FetchSemaphore},
        hash::{
            FileHash,
            infra::{FileHasher, hash_hex_async},
        },
        request_client::{
            BytesProgress, FileDownload, Method, Request, RequestClient, RequestError,
            SavedDownload,
        },
    },
};

/// How long a connection may take to open.
pub const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
/// How long a response may stay silent. Per read, not per request: a large download that keeps
/// flowing is never cut, a stalled one is. Generous, so a poor connection is not mistaken for a
/// dead one; the retry middleware covers the rest.
pub const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(90);

/// Give `builder` timeouts, so a stalled server cannot hold a permit forever.
pub fn with_timeouts(
    builder: reqwest::ClientBuilder,
    connect: std::time::Duration,
    read: std::time::Duration,
) -> reqwest::ClientBuilder {
    builder.connect_timeout(connect).read_timeout(read)
}

/// [`with_timeouts`] with [`CONNECT_TIMEOUT`] and [`READ_TIMEOUT`].
pub fn with_default_timeouts(builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    with_timeouts(builder, CONNECT_TIMEOUT, READ_TIMEOUT)
}

/// The queue a client's requests wait in.
enum Permits {
    Downloads(Arc<FetchSemaphore>),
    Metadata(Arc<ApiSemaphore>),
}

impl Permits {
    async fn acquire(&self) -> Result<tokio::sync::SemaphorePermit<'_>, RequestError> {
        let semaphore = match self {
            Self::Downloads(semaphore) => &semaphore.0,
            Self::Metadata(semaphore) => &semaphore.0,
        };
        Ok(semaphore.acquire().await?)
    }
}

/// A client bound to one queue: build one for the file downloads and one for the metadata
/// fetches, so a long line of downloads never holds a metadata request back and a service
/// cannot pick the wrong queue per request.
pub struct ReqwestClient<PS: ProgressService + ?Sized> {
    progress_service: Arc<PS>,
    client: Arc<reqwest_middleware::ClientWithMiddleware>,
    permits: Permits,
}

impl<PS: ProgressService + ?Sized> ReqwestClient<PS> {
    /// A client whose requests wait for a download permit.
    pub fn for_downloads(
        progress_service: Arc<PS>,
        client: Arc<reqwest_middleware::ClientWithMiddleware>,
        semaphore: Arc<FetchSemaphore>,
    ) -> Self {
        Self {
            progress_service,
            client,
            permits: Permits::Downloads(semaphore),
        }
    }

    /// A client whose requests wait for a metadata permit.
    pub fn for_metadata(
        progress_service: Arc<PS>,
        client: Arc<reqwest_middleware::ClientWithMiddleware>,
        semaphore: Arc<ApiSemaphore>,
    ) -> Self {
        Self {
            progress_service,
            client,
            permits: Permits::Metadata(semaphore),
        }
    }

    async fn send(
        &self,
        method: Method,
        url: &str,
        headers: Option<reqwest::header::HeaderMap>,
        body: Option<Vec<u8>>,
        timeout: Option<std::time::Duration>,
    ) -> Result<Response, RequestError> {
        let mut request = self.client.request(method.into(), url);

        if let Some(timeout) = timeout {
            request = request.timeout(timeout);
        }

        if let Some(body) = body {
            request = request.body(body);
        }

        if let Some(headers) = headers {
            request = request.headers(headers);
        }

        request.send().await.map_err(|e| match e {
            reqwest_middleware::Error::Middleware(error) => RequestError::Middleware(error),
            reqwest_middleware::Error::Reqwest(error) => RequestError::RequestSend(error),
        })
    }

    async fn fetch_chunks(
        &self,
        response: Response,
        progress_bar: (&ProgressBarId, f64),
    ) -> Result<Bytes, RequestError> {
        use futures::StreamExt;

        let Some(total_size) = response.content_length() else {
            return Ok(response.bytes().await?);
        };

        let mut stream = response.bytes_stream();
        let mut bytes = Vec::new();

        let (progress_bar_id, total) = progress_bar;
        // Added, not set: fetches sharing a bar each bring their own size.
        self.progress_service
            .add_progress_bytes_total_safe(progress_bar_id, total_size)
            .await;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(RequestError::RequestSend)?;
            bytes.extend_from_slice(&chunk);

            self.progress_service
                .add_progress_bytes_safe(progress_bar_id, chunk.len() as u64)
                .await;

            #[allow(clippy::cast_precision_loss)]
            let progress = (chunk.len() as f64 / total_size as f64) * total;

            self.progress_service
                .emit_progress_safe(progress_bar_id, progress, None)
                .await;
        }

        Ok(bytes.into())
    }

    async fn verify_hash(&self, bytes: Bytes, expected: &FileHash) -> Result<(), RequestError> {
        let actual = hash_hex_async(expected.algorithm(), bytes)
            .await
            .map_err(|err| RequestError::Middleware(err.into()))?;
        check_hash(expected, actual)
    }
}

fn check_hash(expected: &FileHash, actual: String) -> Result<(), RequestError> {
    if expected.matches(&actual) {
        Ok(())
    } else {
        Err(RequestError::Hash {
            actual,
            expected: expected.hex().to_owned(),
        })
    }
}

/// Reject any non-`2xx` answer before its body is read, so an error page never travels on as data.
fn ensure_success(response: &Response, url: &str) -> Result<(), RequestError> {
    let status = response.status();
    if status.is_success() {
        Ok(())
    } else {
        Err(RequestError::Status {
            status: status.as_u16(),
            url: url.to_owned(),
        })
    }
}

fn io_error(path: &Path) -> impl FnOnce(std::io::Error) -> RequestError + '_ {
    move |source| RequestError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[async_trait]
impl<PS: ProgressService + ?Sized> RequestClient for ReqwestClient<PS> {
    async fn fetch_bytes(&self, request: Request) -> Result<Bytes, RequestError> {
        self.fetch_bytes_with_progress(request, None).await
    }

    async fn fetch_bytes_with_progress(
        &self,
        request: Request,
        progress_bar: Option<(&ProgressBarId, f64)>,
    ) -> Result<Bytes, RequestError> {
        let Request {
            method,
            url,
            headers,
            hash,
            body,
            timeout,
        } = request;

        let _permit = self.permits.acquire().await?;

        let response = self.send(method, &url, headers, body, timeout).await?;
        ensure_success(&response, &url)?;

        let bytes = match progress_bar {
            Some(progress_bar) => self.fetch_chunks(response, progress_bar).await,
            None => response.bytes().await.map_err(Into::into),
        }?;

        if let Some(expected) = hash {
            self.verify_hash(bytes.clone(), &expected).await?;
        }

        Ok(bytes)
    }

    async fn download_to_file(
        &self,
        request: Request,
        destination: &Path,
        max_size: u64,
        on_bytes: Option<BytesProgress<'_>>,
    ) -> Result<u64, RequestError> {
        let Request {
            method,
            url,
            headers,
            hash,
            body,
            timeout,
        } = request;

        let _permit = self.permits.acquire().await?;

        let response = self.send(method, &url, headers, body, timeout).await?;
        ensure_success(&response, &url)?;

        save_body(response, destination, max_size, hash.as_ref(), on_bytes).await
    }

    async fn download_to_file_conditional(
        &self,
        request: Request,
        destination: &Path,
        max_size: u64,
    ) -> Result<FileDownload, RequestError> {
        let Request {
            method,
            url,
            headers,
            hash,
            body,
            timeout,
        } = request;

        let _permit = self.permits.acquire().await?;

        let response = self.send(method, &url, headers, body, timeout).await?;
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            return Ok(FileDownload::NotModified);
        }
        ensure_success(&response, &url)?;

        let header = |name: reqwest::header::HeaderName| {
            response
                .headers()
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned)
        };
        let content_type = header(reqwest::header::CONTENT_TYPE);
        let etag = header(reqwest::header::ETAG);
        let last_modified = header(reqwest::header::LAST_MODIFIED);

        let size = save_body(response, destination, max_size, hash.as_ref(), None).await?;

        Ok(FileDownload::Saved(SavedDownload {
            size,
            content_type,
            etag,
            last_modified,
        }))
    }
}

/// Stream the body of a successful `response` to `destination`, as
/// [`RequestClient::download_to_file`] describes: all or nothing.
async fn save_body(
    response: Response,
    destination: &Path,
    max_size: u64,
    hash: Option<&FileHash>,
    on_bytes: Option<BytesProgress<'_>>,
) -> Result<u64, RequestError> {
    use futures::StreamExt;

    let parent = destination.parent().ok_or_else(|| RequestError::Io {
        path: destination.to_path_buf(),
        source: std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "destination has no parent directory",
        ),
    })?;

    if let Some(length) = response.content_length()
        && length > max_size
    {
        return Err(RequestError::TooLarge { limit: max_size });
    }

    // A temporary file next to the destination, so that the last step is a rename within
    // one directory and a partial download never shows up under the final name. Until
    // `persist` succeeds, dropping `temp_path` — on any early return below — deletes it.
    let (file, temp_path) = tempfile::Builder::new()
        .prefix(".download-")
        .suffix(".part")
        .tempfile_in(parent)
        .map_err(io_error(parent))?
        .into_parts();
    let mut file = tokio::fs::File::from_std(file);

    let mut hasher = hash.map(|hash| FileHasher::new(hash.algorithm()));
    let mut size: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(RequestError::RequestSend)?;

        size += chunk.len() as u64;
        if size > max_size {
            return Err(RequestError::TooLarge { limit: max_size });
        }

        if let Some(hasher) = hasher.as_mut() {
            hasher.update(&chunk);
        }
        file.write_all(&chunk).await.map_err(io_error(&temp_path))?;

        if let Some(on_bytes) = on_bytes {
            on_bytes(chunk.len() as u64);
        }
    }

    file.flush().await.map_err(io_error(&temp_path))?;
    // Close the handle before renaming: Windows refuses to move a file that is still open.
    // `into_std` waits for any write still in flight, so the drop really closes it.
    drop(file.into_std().await);

    if let (Some(expected), Some(hasher)) = (hash, hasher) {
        check_hash(expected, hasher.finalize_hex())?;
    }

    temp_path
        .persist(destination)
        .map_err(|err| io_error(destination)(err.error))?;

    Ok(size)
}

impl From<Method> for reqwest::Method {
    fn from(method: Method) -> Self {
        match method {
            Method::Options => reqwest::Method::OPTIONS,
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Head => reqwest::Method::HEAD,
            Method::Trace => reqwest::Method::TRACE,
            Method::Connect => reqwest::Method::CONNECT,
            Method::Patch => reqwest::Method::PATCH,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use std::net::SocketAddr;

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    use super::*;
    use crate::{
        features::events::{EventError, ProgressEventType},
        shared::request_client::RequestClientExt,
    };

    fn bar() -> ProgressBarId {
        ProgressBarId {
            id: uuid::Uuid::new_v4(),
            progress_storage: None,
            sequencer: None,
        }
    }

    struct NoopProgress;

    #[async_trait]
    impl ProgressService for NoopProgress {
        async fn init_progress(
            &self,
            _event_type: ProgressEventType,
            _total: f64,
            _message: String,
        ) -> Result<ProgressBarId, EventError> {
            Ok(bar())
        }

        async fn init_or_edit_progress(
            &self,
            progress_bar_id: Option<ProgressBarId>,
            _event_type: ProgressEventType,
            _total: f64,
            _message: String,
        ) -> Result<ProgressBarId, EventError> {
            Ok(progress_bar_id.unwrap_or_else(bar))
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

    /// Records the bytes reported on any bar.
    #[derive(Default)]
    struct RecordingBytes {
        bytes: std::sync::Mutex<Vec<u64>>,
        total: std::sync::Mutex<u64>,
    }

    #[async_trait]
    impl ProgressService for RecordingBytes {
        async fn init_progress(
            &self,
            event_type: ProgressEventType,
            total: f64,
            message: String,
        ) -> Result<ProgressBarId, EventError> {
            NoopProgress.init_progress(event_type, total, message).await
        }

        async fn init_or_edit_progress(
            &self,
            progress_bar_id: Option<ProgressBarId>,
            event_type: ProgressEventType,
            total: f64,
            message: String,
        ) -> Result<ProgressBarId, EventError> {
            NoopProgress
                .init_or_edit_progress(progress_bar_id, event_type, total, message)
                .await
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

        async fn add_progress_bytes(
            &self,
            _progress_bar_id: &ProgressBarId,
            bytes: u64,
        ) -> Result<(), EventError> {
            self.bytes.lock().unwrap().push(bytes);
            Ok(())
        }

        async fn add_progress_bytes_total(
            &self,
            _progress_bar_id: &ProgressBarId,
            bytes: u64,
        ) -> Result<(), EventError> {
            *self.total.lock().unwrap() += bytes;
            Ok(())
        }
    }

    fn client() -> ReqwestClient<NoopProgress> {
        client_with(Arc::new(NoopProgress))
    }

    fn client_with<PS: ProgressService>(progress: Arc<PS>) -> ReqwestClient<PS> {
        ReqwestClient::for_downloads(
            progress,
            Arc::new(
                reqwest_middleware::ClientBuilder::new(
                    reqwest::Client::builder()
                        .no_proxy()
                        .build()
                        .expect("test client should build"),
                )
                .build(),
            ),
            Arc::new(FetchSemaphore(tokio::sync::Semaphore::new(1))),
        )
    }

    /// Serve every connection one raw response, then close.
    async fn serve(status_line: &str, body: &'static [u8]) -> SocketAddr {
        serve_with_headers(status_line, "", body).await
    }

    /// As [`serve`], with `headers` (each line ending in `\r\n`) added to the response head.
    async fn serve_with_headers(
        status_line: &str,
        headers: &str,
        body: &'static [u8],
    ) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test server should bind");
        let addr = listener.local_addr().expect("test server should have addr");
        let head = format!(
            "HTTP/1.1 {status_line}\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );

        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let head = head.clone();
                tokio::spawn(async move {
                    // Read the request head, so closing does not reset a client still writing.
                    let mut received = Vec::new();
                    let mut buf = [0_u8; 1024];
                    while !received.windows(4).any(|window| window == b"\r\n\r\n") {
                        match stream.read(&mut buf).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => received.extend_from_slice(&buf[..n]),
                        }
                    }
                    let _ = stream.write_all(head.as_bytes()).await;
                    let _ = stream.write_all(body).await;
                    let _ = stream.shutdown().await;
                });
            }
        });

        addr
    }

    fn url(addr: SocketAddr) -> String {
        format!("http://{addr}/file")
    }

    fn plain_http() -> Arc<reqwest_middleware::ClientWithMiddleware> {
        Arc::new(reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build())
    }

    #[tokio::test]
    async fn should_wait_for_a_download_permit_when_a_download_client_fetches() {
        let addr = serve("200 OK", b"payload").await;
        let downloads = Arc::new(FetchSemaphore(tokio::sync::Semaphore::new(1)));
        let client =
            ReqwestClient::for_downloads(Arc::new(NoopProgress), plain_http(), downloads.clone());

        let held = downloads.0.acquire().await.unwrap();
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(300),
            client.fetch_bytes(Request::get(url(addr))),
        )
        .await;
        assert!(result.is_err(), "the fetch should still be queued");

        drop(held);
        let bytes = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.fetch_bytes(Request::get(url(addr))),
        )
        .await
        .expect("the fetch should run once a permit is free")
        .unwrap();
        assert_eq!(&bytes[..], b"payload");
    }

    #[tokio::test]
    async fn should_fetch_when_every_download_permit_is_taken_and_the_client_is_for_metadata() {
        let addr = serve("200 OK", b"payload").await;
        let downloads = Arc::new(FetchSemaphore(tokio::sync::Semaphore::new(1)));
        let api = Arc::new(ApiSemaphore(tokio::sync::Semaphore::new(1)));
        let _held = downloads.0.acquire().await.unwrap();
        let client = ReqwestClient::for_metadata(Arc::new(NoopProgress), plain_http(), api);

        let bytes = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.fetch_bytes(Request::get(url(addr))),
        )
        .await
        .expect("a metadata fetch must not wait for the download permits")
        .unwrap();
        assert_eq!(&bytes[..], b"payload");
    }

    #[tokio::test]
    async fn should_wait_for_a_metadata_permit_when_a_metadata_client_fetches() {
        let addr = serve("200 OK", b"payload").await;
        let api = Arc::new(ApiSemaphore(tokio::sync::Semaphore::new(1)));
        let client = ReqwestClient::for_metadata(Arc::new(NoopProgress), plain_http(), api.clone());

        let _held = api.0.acquire().await.unwrap();
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(300),
            client.fetch_bytes(Request::get(url(addr))),
        )
        .await;
        assert!(result.is_err(), "the fetch should still be queued");
    }

    #[tokio::test]
    async fn should_wait_for_a_download_permit_when_downloading() {
        let addr = serve("200 OK", b"payload").await;
        let downloads = Arc::new(FetchSemaphore(tokio::sync::Semaphore::new(1)));
        let _held = downloads.0.acquire().await.unwrap();
        let client = ReqwestClient::for_downloads(
            Arc::new(NoopProgress),
            Arc::new(reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build()),
            downloads.clone(),
        );
        let dir = tempfile::tempdir().unwrap();

        let result = tokio::time::timeout(
            std::time::Duration::from_millis(300),
            client.download_to_file(
                Request::get(url(addr)),
                &dir.path().join("file"),
                1 << 20,
                None,
            ),
        )
        .await;

        assert!(result.is_err(), "the download should still be queued");
    }

    /// Accept connections and answer nothing, as a hung server does.
    async fn serve_silence() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let mut open = Vec::new();
            while let Ok((stream, _)) = listener.accept().await {
                open.push(stream);
            }
        });
        addr
    }

    #[tokio::test]
    async fn should_fail_when_the_server_stays_silent_past_the_read_timeout() {
        let addr = serve_silence().await;
        let client = ReqwestClient::for_downloads(
            Arc::new(NoopProgress),
            Arc::new(
                reqwest_middleware::ClientBuilder::new(
                    with_timeouts(
                        reqwest::Client::builder().no_proxy(),
                        std::time::Duration::from_secs(5),
                        std::time::Duration::from_millis(200),
                    )
                    .build()
                    .unwrap(),
                )
                .build(),
            ),
            Arc::new(FetchSemaphore(tokio::sync::Semaphore::new(1))),
        );

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.fetch_bytes(Request::get(url(addr))),
        )
        .await
        .expect("the read timeout must end the request");

        assert!(
            matches!(result, Err(RequestError::RequestSend(_))),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn should_return_the_body_when_a_fetch_succeeds() {
        let addr = serve("200 OK", b"payload").await;

        let bytes = client().fetch_bytes(Request::get(url(addr))).await.unwrap();

        assert_eq!(&bytes[..], b"payload");
    }

    #[rstest]
    #[case::not_found("404 Not Found", 404, b"<html>not found</html>")]
    #[case::server_fails("500 Internal Server Error", 500, b"oops")]
    #[tokio::test]
    async fn should_reject_the_fetch_when_the_status_is_not_ok(
        #[case] status_line: &str,
        #[case] status: u16,
        #[case] body: &'static [u8],
    ) {
        let addr = serve(status_line, body).await;
        let request_url = url(addr);

        let result = client().fetch_bytes(Request::get(&request_url)).await;

        assert!(matches!(
            result,
            Err(RequestError::Status { status: got, url }) if got == status && url == request_url
        ));
    }

    #[tokio::test]
    async fn should_reject_a_failed_status_before_the_progress_pipeline_when_downloading() {
        let addr = serve("404 Not Found", b"missing").await;
        let progress_bar = bar();

        let result = client()
            .fetch_bytes_with_progress(Request::get(url(addr)), Some((&progress_bar, 100.0)))
            .await;

        assert!(matches!(
            result,
            Err(RequestError::Status { status: 404, .. })
        ));
    }

    #[tokio::test]
    async fn should_report_the_bytes_of_a_fetch_with_progress() {
        const BODY: &[u8] = &[7_u8; 200_000];
        let addr = serve("200 OK", BODY).await;
        let progress = Arc::new(RecordingBytes::default());
        let progress_bar = bar();

        let bytes = client_with(progress.clone())
            .fetch_bytes_with_progress(Request::get(url(addr)), Some((&progress_bar, 100.0)))
            .await
            .unwrap();

        assert_eq!(bytes.len(), BODY.len());
        assert_eq!(*progress.total.lock().unwrap(), BODY.len() as u64);
        let reported = progress.bytes.lock().unwrap();
        assert!(!reported.is_empty());
        assert_eq!(reported.iter().sum::<u64>(), BODY.len() as u64);
    }

    #[tokio::test]
    async fn should_report_a_failed_status_not_a_parse_error_when_fetching_json() {
        let addr = serve("404 Not Found", b"not json at all").await;

        let result = client()
            .fetch_json::<serde_json::Value>(Request::get(url(addr)))
            .await;

        assert!(matches!(
            result,
            Err(RequestError::Status { status: 404, .. })
        ));
    }

    #[tokio::test]
    async fn should_parse_json_when_a_fetch_succeeds() {
        let addr = serve("200 OK", br#"{"a":1}"#).await;

        let value = client()
            .fetch_json::<serde_json::Value>(Request::get(url(addr)))
            .await
            .unwrap();

        assert_eq!(value["a"], 1);
    }

    fn dir_is_empty(dir: &Path) -> bool {
        std::fs::read_dir(dir).unwrap().next().is_none()
    }

    #[tokio::test]
    async fn should_report_every_byte_and_end_at_its_size_when_downloading() {
        const BODY: &[u8] = &[7_u8; 200_000];
        let addr = serve("200 OK", BODY).await;
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("file");
        let seen = std::sync::Mutex::new(Vec::<u64>::new());

        let size = client()
            .download_to_file(
                Request::get(url(addr)),
                &destination,
                1 << 20,
                Some(&|bytes| seen.lock().unwrap().push(bytes)),
            )
            .await
            .unwrap();

        let seen = seen.into_inner().unwrap();
        let mut total = 0;
        for bytes in &seen {
            assert!(*bytes > 0);
            total += bytes;
        }
        assert_eq!(size, BODY.len() as u64);
        assert_eq!(total, size);
        assert_eq!(std::fs::read(&destination).unwrap(), BODY);
    }

    #[tokio::test]
    async fn should_leave_nothing_when_the_hash_is_wrong_even_after_progress() {
        let addr = serve("200 OK", b"payload").await;
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("file");
        let wrong = FileHash::parse("sha1", &"0".repeat(40)).unwrap();

        let result = client()
            .download_to_file(
                Request::get(url(addr)).with_hash(wrong),
                &destination,
                1 << 20,
                Some(&|_| {}),
            )
            .await;

        assert!(
            matches!(result, Err(RequestError::Hash { .. })),
            "{result:?}"
        );
        assert!(dir_is_empty(dir.path()));
    }

    /// The server sends half the body and stalls; the download is dropped, as the executor does
    /// to the rest of its downloads when one fails.
    #[tokio::test]
    async fn should_leave_no_partial_file_when_a_download_is_cancelled() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = [0_u8; 1024];
            let _ = stream.read(&mut buf).await;
            let _ = stream
                .write_all(
                    b"HTTP/1.1 200 OK
Content-Length: 100000

",
                )
                .await;
            let _ = stream.write_all(&vec![1_u8; 50_000]).await;
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        });
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("file");
        let received = std::sync::atomic::AtomicU64::new(0);
        let got_bytes = tokio::sync::Notify::new();
        let client = client();
        let on_bytes = |bytes| {
            received.fetch_add(bytes, std::sync::atomic::Ordering::SeqCst);
            got_bytes.notify_one();
        };

        let download = client.download_to_file(
            Request::get(url(addr)),
            &destination,
            1 << 20,
            Some(&on_bytes),
        );
        // Cancelled as soon as the first bytes arrive, not after a guessed delay, so a slow
        // machine cannot cancel it before there is anything to clean up.
        let outcome = tokio::time::timeout(std::time::Duration::from_secs(30), async {
            tokio::select! {
                result = download => Some(result),
                () = got_bytes.notified() => None,
            }
        })
        .await;

        assert!(
            matches!(outcome, Ok(None)),
            "the stalled download should be cancelled after its first bytes: {outcome:?}"
        );
        assert!(received.load(std::sync::atomic::Ordering::SeqCst) > 0);
        assert!(dir_is_empty(dir.path()));
    }

    #[tokio::test]
    async fn should_report_type_and_validators_when_a_conditional_download_saves() {
        let addr = serve_with_headers(
            "200 OK",
            "Content-Type: image/png\r\nETag: \"abc\"\r\nLast-Modified: Wed, 21 Oct 2015 07:28:00 GMT\r\n",
            b"png",
        )
        .await;
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("file");

        let outcome = client()
            .download_to_file_conditional(Request::get(url(addr)), &destination, 1 << 20)
            .await
            .unwrap();

        assert_eq!(
            outcome,
            FileDownload::Saved(SavedDownload {
                size: 3,
                content_type: Some("image/png".into()),
                etag: Some("\"abc\"".into()),
                last_modified: Some("Wed, 21 Oct 2015 07:28:00 GMT".into()),
            })
        );
        assert_eq!(std::fs::read(&destination).unwrap(), b"png");
    }

    #[tokio::test]
    async fn should_leave_the_file_when_a_conditional_download_is_not_modified() {
        let addr = serve("304 Not Modified", b"").await;
        let dir = tempfile::tempdir().unwrap();
        let destination = dir.path().join("file");
        std::fs::write(&destination, b"old").unwrap();

        let outcome = client()
            .download_to_file_conditional(Request::get(url(addr)), &destination, 1 << 20)
            .await
            .unwrap();

        assert_eq!(outcome, FileDownload::NotModified);
        assert_eq!(std::fs::read(&destination).unwrap(), b"old");
    }

    #[tokio::test]
    async fn should_reject_a_conditional_download_when_the_status_fails() {
        let addr = serve("404 Not Found", b"missing").await;
        let dir = tempfile::tempdir().unwrap();

        let result = client()
            .download_to_file_conditional(
                Request::get(url(addr)),
                &dir.path().join("file"),
                1 << 20,
            )
            .await;

        assert!(matches!(
            result,
            Err(RequestError::Status { status: 404, .. })
        ));
        assert!(dir_is_empty(dir.path()));
    }
}
