use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

/// A plain HTTP GET request issued by a plugin through the `http_get` host function.
///
/// Unlike Extism's built-in `http_request`, the host checks **every** redirect hop as if it
/// were the first request (`allowed_hosts`, public addresses, no `https` → `http`) and follows
/// at most 5 of them. The request headers are not sent on to another origin.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpRequestDto {
    /// Absolute URL. Only the `http` and `https` schemes are accepted.
    pub url: String,
    /// Extra request headers. Hop-by-hop and connection-control headers are rejected
    /// by the host (see `HttpRequestDto` documentation in the plugin API README).
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

/// A JSON POST request issued by a plugin through the `http_post_json` host function.
///
/// Same network policy as [`HttpRequestDto`] (host `allowed_hosts`, no non-public
/// addresses), except that redirects are never followed: any 3xx is an error. The host always
/// sends `Content-Type: application/json`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpPostJsonRequestDto {
    /// Absolute URL. Only the `http` and `https` schemes are accepted.
    pub url: String,
    /// Extra request headers. Unlike `http_get`, only a fixed allowlist is honoured
    /// (`accept`, `accept-language`, `authorization`, `user-agent`, `x-api-key`);
    /// anything else, including `Content-Type`, is silently dropped.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Already-serialized JSON document. The host rejects a body that is not valid JSON.
    pub body: String,
}

/// Response returned by the `http_get` and `http_post_json` host functions.
///
/// The body travels as a msgpack `bin`, not as an array of numbers: a plugin decodes it with a
/// copy instead of an instruction per byte, which is what keeps a megabyte-sized body within
/// the plugin's fuel.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpResponseDto {
    /// HTTP status code of the final response, after any redirects the host followed.
    pub status: u16,
    /// Response headers, lowercased. Repeated headers are collapsed to the last value.
    pub headers: HashMap<String, String>,
    /// Response body. Truncation never happens silently: a body over the host limit
    /// fails the whole call.
    #[serde_as(as = "Bytes")]
    pub body: Vec<u8>,
}
