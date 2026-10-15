//! `http_get` — the host-controlled HTTP client exposed to plugins.
//!
//! Extism's built-in `http_request` checks `allowed_hosts` only for the initial URL and then
//! follows redirects unchecked, so a permitted host can bounce a plugin to any other host. The
//! requests here instead:
//!
//! * check the host against the plugin's `allowed_hosts` in the host;
//! * refuse any non-public resolved address (loopback, private, link-local such as
//!   `169.254.169.254`) — SSRF;
//! * pin the connection to the addresses that were checked, so a second DNS answer cannot swap
//!   in a private one (DNS rebinding);
//! * follow at most `max_redirects` redirects, GET only, each hop vetted like the first request
//!   before anything is sent to it; `https` → `http` is refused, and the plugin's headers are
//!   dropped once the chain leaves its origin. Any other method treats a 3xx as an error;
//! * cap the response size and the total request time, redirects included.
//!
//! reqwest never follows a redirect itself: [`VettedHops`] runs the hop loop, and the client's
//! only DNS is [`PinnedResolver`], so a name that was not vetted cannot even be resolved.
//!
//! **Behind a proxy** (the system proxy settings, as the application's client uses them) the
//! proxy resolves and connects, so addresses can be neither checked nor pinned; everything else
//! still holds, including the address check of an IP-literal host. The accepted residual risk:
//! an allowlisted host whose DNS points at an internal address reaches the proxy's network,
//! which takes control of that host's DNS, far more than a single redirect.

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use aether_core_plugin_api::v0::{HttpPostJsonRequestDto, HttpRequestDto, HttpResponseDto};
use extism::host_fn;
use extism_convert::Msgpack;
use futures::future::BoxFuture;
use hyper_util::client::proxy::matcher::Matcher as ProxyMatcher;
use reqwest::redirect;
use reqwest_middleware::{ClientWithMiddleware, Middleware};
use url::Url;

use super::super::{super::mappers::to_extism_res, PluginContext, block_on};
pub(super) use policy::policy_error;
use policy::{PinnedResolver, VettedHops};
// The allowlist match and the address policy are the domain's: giving a host to a plugin
// (`GrantPluginHostAccessUseCase`) is decided by the same rules its requests are held to.
pub(crate) use crate::features::plugins::{is_blocked_ip, is_host_allowed};
use crate::shared::limits::PluginSandboxLimits;

mod policy;

/// Total time budget for one `http_get`, including connect and body download.
pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Time budget for establishing the connection alone.
pub(crate) const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Longest a connection may stay silent mid-response. Only matters for calls whose total
/// budget is longer than this, i.e. `download_to_cache`.
pub(crate) const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// The only headers `http_post_json` honours. Everything else is dropped, and
/// `Content-Type` is always set by the host.
const POST_ALLOWED_HEADERS: &[&str] = &[
    "accept",
    "accept-language",
    "authorization",
    "user-agent",
    "x-api-key",
];

/// Headers `http_get` plugins may not set: they either control the connection itself or
/// decide which virtual host the request lands on.
const FORBIDDEN_HEADERS: &[&str] = &[
    "connection",
    "content-length",
    "host",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

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

    #[error("Host `{host}` resolves to non-public address {ip}, which is not allowed")]
    BlockedAddress { host: String, ip: IpAddr },

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

impl From<PluginHttpError> for crate::ErrorKind {
    fn from(value: PluginHttpError) -> Self {
        Self::CoreError(value.to_string())
    }
}

// ── Address filtering ──

/// Decides whether a resolved address must be refused.
///
/// Always [`is_blocked_ip`] in production; tests substitute a permissive filter so that a
/// server on loopback can be exercised without weakening the real policy.
pub(crate) type AddressFilter = fn(IpAddr) -> bool;

/// Resolves a host name to the addresses the filter then judges.
///
/// Always [`system_lookup`] in production; tests substitute a fixed answer — a DNS that hands
/// out Clash fake-ip addresses, say — without a real resolver behind it.
pub(super) type HostLookup =
    fn(String, u16) -> BoxFuture<'static, std::io::Result<Vec<SocketAddr>>>;

/// The operating system's resolver.
pub(super) fn system_lookup(
    host: String,
    port: u16,
) -> BoxFuture<'static, std::io::Result<Vec<SocketAddr>>> {
    Box::pin(async move {
        // `Url::host_str` keeps the brackets of an IPv6 literal, which the resolver rejects.
        let bare = host
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
            .unwrap_or(&host);
        Ok(tokio::net::lookup_host((bare, port)).await?.collect())
    })
}

// ── Testable business logic ──

/// Validate the request and return the URL plus the host to check against the allowlist.
fn parse_url(raw_url: &str) -> Result<(Url, String), PluginHttpError> {
    let url = Url::parse(raw_url).map_err(|e| PluginHttpError::InvalidUrl {
        url: raw_url.to_owned(),
        reason: e.to_string(),
    })?;

    let scheme = url.scheme().to_owned();
    if scheme != "http" && scheme != "https" {
        return Err(PluginHttpError::UnsupportedScheme { scheme });
    }

    let host = url
        .host_str()
        .ok_or_else(|| PluginHttpError::MissingHost {
            url: raw_url.to_owned(),
        })?
        .to_owned();

    Ok((url, host))
}

fn build_headers(
    headers: &HashMap<String, String>,
) -> Result<reqwest::header::HeaderMap, PluginHttpError> {
    let mut header_map = reqwest::header::HeaderMap::with_capacity(headers.len());

    for (name, value) in headers {
        let lowercased = name.to_ascii_lowercase();
        if FORBIDDEN_HEADERS.contains(&lowercased.as_str()) {
            return Err(PluginHttpError::ForbiddenHeader { name: lowercased });
        }

        let header_name =
            reqwest::header::HeaderName::from_bytes(lowercased.as_bytes()).map_err(|e| {
                PluginHttpError::InvalidHeader {
                    name: lowercased.clone(),
                    reason: e.to_string(),
                }
            })?;
        let header_value = reqwest::header::HeaderValue::from_str(value).map_err(|e| {
            PluginHttpError::InvalidHeader {
                name: lowercased.clone(),
                reason: e.to_string(),
            }
        })?;

        header_map.insert(header_name, header_value);
    }

    Ok(header_map)
}

fn send_error(url: &Url, err: reqwest_middleware::Error) -> PluginHttpError {
    let reason = match err {
        reqwest_middleware::Error::Middleware(err) => match policy_error(err) {
            Ok(policy) => return policy,
            Err(err) => err.to_string(),
        },
        reqwest_middleware::Error::Reqwest(err) => err.to_string(),
    };

    PluginHttpError::RequestFailed {
        url: url.to_string(),
        reason,
    }
}

async fn read_limited_body(
    mut response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, PluginHttpError> {
    let url = response.url().to_string();

    if let Some(length) = response.content_length()
        && length > max_bytes as u64
    {
        return Err(PluginHttpError::ResponseTooLarge { limit: max_bytes });
    }

    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| PluginHttpError::RequestFailed {
            url: url.clone(),
            reason: e.to_string(),
        })?
    {
        if body.len() + chunk.len() > max_bytes {
            return Err(PluginHttpError::ResponseTooLarge { limit: max_bytes });
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

/// Shared first step of every request: the URL must be http(s) and its host must be in
/// the plugin's `allowed_hosts`. Nothing touches the network before this passes.
pub(super) fn check_target(
    raw_url: &str,
    allowed_hosts: &[String],
) -> Result<(Url, String), PluginHttpError> {
    let (url, host) = parse_url(raw_url)?;

    if !is_host_allowed(&host, allowed_hosts) {
        return Err(PluginHttpError::HostNotAllowed { host });
    }

    Ok((url, host))
}

/// Where the proxy settings of a plugin's requests come from.
pub(crate) enum ProxySource {
    /// What the application's shared client uses: the proxy environment variables, else the
    /// OS settings. reqwest's default and [`ProxyMatcher::from_system`] read the same source.
    System,
    /// No proxy at all.
    #[cfg(test)]
    Direct,
    /// Every request through this proxy.
    #[cfg(test)]
    Fixed(String),
}

impl ProxySource {
    fn matcher(&self) -> ProxyMatcher {
        match self {
            Self::System => ProxyMatcher::from_system(),
            #[cfg(test)]
            Self::Direct => ProxyMatcher::builder().build(),
            #[cfg(test)]
            Self::Fixed(proxy) => ProxyMatcher::builder().all(proxy.as_str()).build(),
        }
    }

    fn configure(&self, builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
        match self {
            // reqwest's default reads exactly what `ProxyMatcher::from_system` reads.
            Self::System => builder,
            #[cfg(test)]
            Self::Direct => builder.no_proxy(),
            #[cfg(test)]
            Self::Fixed(proxy) => builder
                .no_proxy()
                .proxy(reqwest::Proxy::all(proxy).expect("test proxy URL should parse")),
        }
    }
}

/// Shared second step: the client every host-side request a plugin causes goes out through,
/// so the address policy and the redirect policy cannot drift between host functions.
///
/// `total_timeout` bounds one whole chain of hops, body included. `outer` (a retry policy, say)
/// wraps the hop loop, so a retry starts over from the first URL and vets it all again. A hop
/// sent through a proxy is vetted the relaxed way described in the module docs.
pub(super) fn vetted_client(
    allowed_hosts: &[String],
    lookup: HostLookup,
    is_blocked: AddressFilter,
    proxy: &ProxySource,
    total_timeout: Duration,
    outer: Option<Arc<dyn Middleware>>,
    max_redirects: usize,
) -> Result<ClientWithMiddleware, PluginHttpError> {
    let resolver = Arc::new(PinnedResolver::default());
    let build_failed = |e: reqwest::Error| PluginHttpError::RequestFailed {
        url: String::new(),
        reason: e.to_string(),
    };

    crate::shared::tls::install_crypto_provider();
    let builder = reqwest::Client::builder()
        // Redirects are followed only by `VettedHops`, one checked hop at a time.
        .redirect(redirect::Policy::none())
        .referer(false)
        .dns_resolver(resolver.clone())
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        .timeout(total_timeout);
    let client = proxy.configure(builder).build().map_err(build_failed)?;

    let hops = VettedHops {
        allowed_hosts: allowed_hosts.to_vec(),
        lookup,
        is_blocked,
        proxies: proxy.matcher(),
        resolver,
        total_timeout,
        max_redirects,
    };

    let mut builder = reqwest_middleware::ClientBuilder::new(client);
    if let Some(outer) = outer {
        builder = builder.with_arc(outer);
    }

    Ok(builder.with(hops).build())
}

/// The whole `http_get` pipeline, parameterised by the address filter and the proxy source so
/// that tests can point it at a loopback server without relaxing the production policy.
async fn perform_get(
    request: &HttpRequestDto,
    allowed_hosts: &[String],
    is_blocked: AddressFilter,
    proxy: &ProxySource,
    limits: PluginSandboxLimits,
) -> Result<HttpResponseDto, PluginHttpError> {
    let (url, _) = check_target(&request.url, allowed_hosts)?;
    let headers = build_headers(&request.headers)?;

    send_checked(
        reqwest::Method::GET,
        url,
        headers,
        None,
        allowed_hosts,
        is_blocked,
        proxy,
        limits,
    )
    .await
}

/// Keep only [`POST_ALLOWED_HEADERS`]; everything else is dropped.
fn filter_post_headers(
    headers: &HashMap<String, String>,
) -> Result<reqwest::header::HeaderMap, PluginHttpError> {
    let permitted: HashMap<String, String> = headers
        .iter()
        .filter(|(name, _)| POST_ALLOWED_HEADERS.contains(&name.to_ascii_lowercase().as_str()))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect();

    build_headers(&permitted)
}

fn validate_json_body(body: &str, max_bytes: usize) -> Result<(), PluginHttpError> {
    if body.len() > max_bytes {
        return Err(PluginHttpError::RequestBodyTooLarge { limit: max_bytes });
    }

    serde_json::from_str::<serde::de::IgnoredAny>(body)
        .map(|_| ())
        .map_err(|e| PluginHttpError::InvalidJsonBody {
            reason: e.to_string(),
        })
}

/// The whole `http_post_json` pipeline; same test seams as [`perform_get`].
async fn perform_post_json(
    request: &HttpPostJsonRequestDto,
    allowed_hosts: &[String],
    is_blocked: AddressFilter,
    proxy: &ProxySource,
    limits: PluginSandboxLimits,
) -> Result<HttpResponseDto, PluginHttpError> {
    let (url, _) = check_target(&request.url, allowed_hosts)?;
    let mut headers = filter_post_headers(&request.headers)?;
    validate_json_body(&request.body, limits.max_request_body_bytes)?;

    headers.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    send_checked(
        reqwest::Method::POST,
        url,
        headers,
        Some(request.body.clone()),
        allowed_hosts,
        is_blocked,
        proxy,
        limits,
    )
    .await
}

/// Send through a [`vetted_client`] and read a size-limited response. Shared by `http_get` and
/// `http_post_json`, so the network policy cannot drift between them. `url` must come from
/// [`check_target`].
// Every argument is a distinct part of the one request policy; grouping them would add a type
// used only here.
#[allow(clippy::too_many_arguments)]
async fn send_checked(
    method: reqwest::Method,
    url: Url,
    headers: reqwest::header::HeaderMap,
    body: Option<String>,
    allowed_hosts: &[String],
    is_blocked: AddressFilter,
    proxy: &ProxySource,
    limits: PluginSandboxLimits,
) -> Result<HttpResponseDto, PluginHttpError> {
    let client = vetted_client(
        allowed_hosts,
        system_lookup,
        is_blocked,
        proxy,
        REQUEST_TIMEOUT,
        None,
        limits.max_redirects,
    )?;

    let mut builder = client.request(method, url.clone()).headers(headers);
    if let Some(body) = body {
        builder = builder.body(body);
    }

    let response = builder.send().await.map_err(|err| send_error(&url, err))?;

    // A 3xx that reaches this point was not followed: any 3xx to a POST, or one a GET does not
    // follow (`304`, …). It is not a response the plugin can use.
    let status = response.status();
    if status.is_redirection() {
        return Err(PluginHttpError::RedirectNotAllowed {
            status: status.as_u16(),
            url: response.url().to_string(),
        });
    }

    let response_headers = response
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_owned(), value.to_owned()))
        })
        .collect();

    let body = read_limited_body(response, limits.max_response_bytes).await?;

    Ok(HttpResponseDto {
        status: status.as_u16(),
        headers: response_headers,
        body,
    })
}

/// Handle `http_get` — perform an allowlisted GET, every redirect hop vetted, on behalf of a plugin.
pub(crate) async fn handle_http_get(
    plugin_id: &str,
    request: HttpRequestDto,
    allowed_hosts: &[String],
    limits: PluginSandboxLimits,
) -> crate::Result<HttpResponseDto> {
    log::debug!(target: "plugin", "[{plugin_id}]: http_get {}", request.url);

    perform_get(
        &request,
        allowed_hosts,
        is_blocked_ip,
        &ProxySource::System,
        limits,
    )
    .await
    .map_err(|err| {
        log::warn!(target: "plugin", "[{plugin_id}]: http_get failed: {err}");
        crate::ErrorKind::from(err).as_error()
    })
}

/// Handle `http_post_json` — perform an allowlisted, redirect-free JSON POST on behalf of a plugin.
pub(crate) async fn handle_http_post_json(
    plugin_id: &str,
    request: HttpPostJsonRequestDto,
    allowed_hosts: &[String],
    limits: PluginSandboxLimits,
) -> crate::Result<HttpResponseDto> {
    log::debug!(target: "plugin", "[{plugin_id}]: http_post_json {}", request.url);

    perform_post_json(
        &request,
        allowed_hosts,
        is_blocked_ip,
        &ProxySource::System,
        limits,
    )
    .await
    .map_err(|err| {
        log::warn!(target: "plugin", "[{plugin_id}]: http_post_json failed: {err}");
        crate::ErrorKind::from(err).as_error()
    })
}

// ── Extism host function wrappers ──

host_fn!(
pub http_post_json(user_data: PluginContext; request: Msgpack<HttpPostJsonRequestDto>) -> HostResult<HttpResponseDto> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.clone();
    let hosts = ctx.hosts.clone();
    let limits = ctx.limits;
    drop(ctx);

    to_extism_res::<HttpResponseDto>(block_on(async {
        let allowed_hosts = hosts.allowed_hosts(&id).await?;
        handle_http_post_json(&id, request.0, &allowed_hosts, limits).await
    }))
});

host_fn!(
pub http_get(user_data: PluginContext; request: Msgpack<HttpRequestDto>) -> HostResult<HttpResponseDto> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.clone();
    let hosts = ctx.hosts.clone();
    let limits = ctx.limits;
    drop(ctx);

    to_extism_res::<HttpResponseDto>(block_on(async {
        let allowed_hosts = hosts.allowed_hosts(&id).await?;
        handle_http_get(&id, request.0, &allowed_hosts, limits).await
    }))
});

#[cfg(test)]
mod tests;
