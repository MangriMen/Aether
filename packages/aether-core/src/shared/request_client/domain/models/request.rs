use crate::shared::{hash::FileHash, request_client::Method};

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: Option<reqwest::header::HeaderMap>,
    /// Expected digest of the response body. A mismatch fails the request.
    pub hash: Option<FileHash>,
    pub body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(method: Method, url: String) -> Self {
        Self {
            method,
            url,
            headers: None,
            hash: None,
            body: None,
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
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
}
