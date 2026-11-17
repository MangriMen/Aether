//! Why a request the host makes for a plugin failed: `http_get`, `http_post_json`,
//! `download_to_cache` and the downloads of an install plan. Typed all the way to the plugin, so
//! that it can tell a refused host from a network that is down.

use std::{net::IpAddr, path::PathBuf, time::Duration};

use crate::shared::{hash::InvalidFileHash, request_client::RequestError};

#[derive(Debug, Clone, thiserror::Error)]
pub enum PluginHttpError {
    #[error("Invalid URL `{url}`: {reason}")]
    InvalidUrl { url: String, reason: String },

    #[error("URL scheme `{scheme}` is not allowed, only `http` and `https` are supported")]
    UnsupportedScheme { scheme: String },

    #[error("URL `{url}` has no host")]
    MissingHost { url: String },

    #[error("Host `{host}` is not listed in the plugin's allowed_hosts")]
    HostNotAllowed { host: String },

    #[error("Failed to resolve host `{host}`: {reason}")]
    ResolveFailed { host: String, reason: String },

    #[error("Host `{host}` resolved to no addresses")]
    NoAddresses { host: String },

    /// `port` is the one the request was for: what a local host grant for it would name.
    #[error("Host `{host}` resolves to non-public address {ip}, which is not allowed")]
    BlockedAddress { host: String, port: u16, ip: IpAddr },

    #[error("Request body is not valid JSON: {reason}")]
    InvalidJsonBody { reason: String },

    #[error("Request body exceeds the {limit} byte limit")]
    RequestBodyTooLarge { limit: usize },

    #[error("Header `{name}` may not be set by a plugin")]
    ForbiddenHeader { name: String },

    #[error("Invalid header `{name}`: {reason}")]
    InvalidHeader { name: String, reason: String },

    #[error("Redirect with status {status} from `{url}` is not followed")]
    RedirectNotAllowed { status: u16, url: String },

    #[error("Redirect with status {status} from `{url}` has no usable Location: {reason}")]
    InvalidRedirect {
        status: u16,
        url: String,
        reason: String,
    },

    #[error("Redirect from `{from}` to `{to}` would downgrade https to http")]
    SchemeDowngrade { from: String, to: String },

    #[error("Too many redirects: more than {limit}")]
    TooManyRedirects { limit: usize },

    #[error("Response body exceeds the {limit} byte limit")]
    ResponseTooLarge { limit: usize },

    #[error("Request to `{url}` failed: {reason}")]
    RequestFailed { url: String, reason: String },
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum PluginDownloadError {
    #[error("Invalid hash: {0}")]
    InvalidHash(#[from] InvalidFileHash),

    #[error(transparent)]
    Http(#[from] PluginHttpError),

    #[error("Download failed: {0}")]
    Request(#[from] RequestError),

    #[error("Plugin cache path `{}` cannot be used: {reason}", path.display())]
    UnusableCache { path: PathBuf, reason: String },

    #[error("Download did not finish within {} seconds", .0.as_secs())]
    OutOfTime(Duration),
}
