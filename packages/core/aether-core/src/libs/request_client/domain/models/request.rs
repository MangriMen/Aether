use crate::libs::request_client::Method;

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: Option<reqwest::header::HeaderMap>,
    pub sha1: Option<String>,
    pub body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(method: Method, url: String) -> Self {
        Self {
            method,
            url,
            headers: None,
            sha1: None,
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
    pub fn with_sha1(mut self, sha1: impl Into<String>) -> Self {
        self.sha1 = Some(sha1.into());
        self
    }

    #[must_use]
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
}
