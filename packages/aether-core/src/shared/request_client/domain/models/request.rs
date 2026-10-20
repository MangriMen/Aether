use std::time::Duration;

use crate::shared::{hash::FileHash, request_client::Method};

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: Option<reqwest::header::HeaderMap>,
    /// Expected digest of the response body. A mismatch fails the request.
    pub hash: Option<FileHash>,
    pub body: Option<Vec<u8>>,
    /// How long the whole request may take, body included, before it fails. `None` — no
    /// limit of its own, only what the client has.
    pub timeout: Option<Duration>,
    /// A file download, bounded by the download limit instead of the metadata one.
    pub bulk: bool,
}

impl Request {
    pub fn new(method: Method, url: String) -> Self {
        Self {
            method,
            url,
            headers: None,
            hash: None,
            body: None,
            timeout: None,
            bulk: false,
        }
    }

    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::Get, url.into())
    }

    pub fn post(url: impl Into<String>) -> Self {
        Self::new(Method::Post, url.into())
    }

    #[must_use]
    pub fn with_headers(mut self, headers: reqwest::header::HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    #[must_use]
    pub fn with_hash(mut self, hash: FileHash) -> Self {
        self.hash = Some(hash);
        self
    }

    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    #[must_use]
    pub fn as_download(mut self) -> Self {
        self.bulk = true;
        self
    }

    #[must_use]
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
}
