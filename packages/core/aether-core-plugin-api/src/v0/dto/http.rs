use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A plain HTTP GET request issued by a plugin through the `http_get` host function.
///
/// Unlike Extism's built-in `http_request`, the host **never follows redirects** for
/// this request: a 3xx response is reported as an error instead of being followed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpRequestDto {
    /// Absolute URL. Only the `http` and `https` schemes are accepted.
    pub url: String,
    /// Extra request headers. Hop-by-hop and connection-control headers are rejected
    /// by the host (see `HttpRequestDto` documentation in the plugin API README).
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

/// Response returned by the `http_get` host function.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpResponseDto {
    /// HTTP status code of the (single, non-redirected) response.
    pub status: u16,
    /// Response headers, lowercased. Repeated headers are collapsed to the last value.
    pub headers: HashMap<String, String>,
    /// Response body. Truncation never happens silently: a body over the host limit
    /// fails the whole call.
    pub body: Vec<u8>,
}
