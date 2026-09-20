//! `http_get` — the host-controlled HTTP client exposed to plugins.
//!
//! Extism's built-in `http_request` checks `allowed_hosts` only for the **initial** URL and
//! then lets the underlying agent follow redirects, so a permitted host can bounce a plugin
//! to any other host (finding N1). This module implements a GET that:
//!
//! * never follows redirects (`redirect::Policy::none()`, 3xx is an error);
//! * re-checks the host against the plugin's `allowed_hosts` **in the host**;
//! * resolves the host and refuses any non-public address (loopback, private,
//!   link-local — including `169.254.169.254`, and friends);
//! * pins the request to the very addresses that were checked, so a second DNS answer
//!   cannot swap in a private address (DNS rebinding);
//! * caps both the response size and the total request time.

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};

use aether_core_plugin_api::v0::{HttpRequestDto, HttpResponseDto};
use extism::host_fn;
use extism_convert::Msgpack;
use reqwest::redirect;
use url::Url;

use crate::shared::execute_async::infra::execute_async;

use super::super::{super::mappers::to_extism_res, PluginContext};

/// Hard cap on the response body a plugin may receive in one call.
pub(crate) const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

/// Total time budget for one `http_get`, including connect and body download.
pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Time budget for establishing the connection alone.
pub(crate) const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Headers a plugin may not set: they either control the connection itself or
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

#[derive(Debug, thiserror::Error)]
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

    #[error("Header `{name}` may not be set by a plugin")]
    ForbiddenHeader { name: String },

    #[error("Invalid header `{name}`: {reason}")]
    InvalidHeader { name: String, reason: String },

    #[error("Redirects are not followed: got status {status} from `{url}`")]
    RedirectNotAllowed { status: u16, url: String },

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

// ── Host allowlist ──

/// Match a host against the plugin's `allowed_hosts`.
///
/// Deliberately mirrors Extism's own check (`extism/src/pdk.rs`): each entry is treated as a
/// glob pattern, and an entry that does not parse as a glob falls back to an exact comparison.
/// Keeping the semantics identical means `http_get` and the built-in `http_request` accept
/// exactly the same set of hosts.
pub(crate) fn is_host_allowed(host: &str, allowed_hosts: &[String]) -> bool {
    allowed_hosts.iter().any(|pattern| {
        glob::Pattern::new(pattern).map_or_else(|_| pattern == host, |glob| glob.matches(host))
    })
}

// ── Address filtering ──

/// Decides whether a resolved address must be refused.
///
/// Always [`is_blocked_ip`] in production; tests substitute a permissive filter so that a
/// server on loopback can be exercised without weakening the real policy.
type AddressFilter = fn(IpAddr) -> bool;

/// Reject everything that is not a public, routable address.
///
/// Fails closed: anything not positively known to be public is blocked.
pub(crate) fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_ipv4(v4),
        IpAddr::V6(v6) => is_blocked_ipv6(v6),
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();

    ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_multicast()
        // 0.0.0.0/8 — "this network"
        || a == 0
        // 100.64.0.0/10 — carrier-grade NAT
        || (a == 100 && (64..128).contains(&b))
        // 192.0.0.0/24 — IETF protocol assignments
        || (a == 192 && b == 0 && c == 0)
        // 198.18.0.0/15 — benchmarking
        || (a == 198 && (b == 18 || b == 19))
        // 240.0.0.0/4 — reserved
        || a >= 240
}

fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    let segments = ip.segments();

    // Checked before the IPv4 conversions below: `::` and `::1` also fall into the
    // deprecated IPv4-compatible range and must not be judged as `0.0.0.0`/`0.0.0.1`.
    if ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_multicast()
        // fc00::/7 — unique local
        || (segments[0] & 0xfe00) == 0xfc00
        // fe80::/10 — link-local unicast
        || (segments[0] & 0xffc0) == 0xfe80
        // 2001:db8::/32 — documentation
        || (segments[0] == 0x2001 && segments[1] == 0x0db8)
    {
        return true;
    }

    // An IPv4 address wearing an IPv6 costume must be judged as IPv4: both the
    // `::ffff:a.b.c.d` (mapped) and the deprecated `::a.b.c.d` (compatible) forms.
    if let Some(v4) = ip.to_ipv4_mapped().or_else(|| ip.to_ipv4()) {
        return is_blocked_ipv4(v4);
    }

    false
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

/// Resolve `host:port` and make sure **every** returned address is public.
///
/// All addresses must pass: a name that resolves to one public and one private address is
/// rejected outright, since we cannot control which one a later connect would pick.
async fn resolve_public_addrs(
    host: &str,
    port: u16,
    is_blocked: AddressFilter,
) -> Result<Vec<SocketAddr>, PluginHttpError> {
    let addrs: Vec<SocketAddr> = tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| PluginHttpError::ResolveFailed {
            host: host.to_owned(),
            reason: e.to_string(),
        })?
        .collect();

    if addrs.is_empty() {
        return Err(PluginHttpError::NoAddresses {
            host: host.to_owned(),
        });
    }

    for addr in &addrs {
        if is_blocked(addr.ip()) {
            return Err(PluginHttpError::BlockedAddress {
                host: host.to_owned(),
                ip: addr.ip(),
            });
        }
    }

    Ok(addrs)
}

fn build_client(host: &str, addrs: &[SocketAddr]) -> Result<reqwest::Client, PluginHttpError> {
    let builder = reqwest::Client::builder()
        // F-9: the shared application client follows up to 10 redirects; this one follows none.
        .redirect(redirect::Policy::none())
        .referer(false)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        // Pin the connection to the addresses we just vetted, so the name cannot be
        // re-resolved to a private address between the check and the connect.
        .resolve_to_addrs(host, addrs);

    builder.build().map_err(|e| PluginHttpError::RequestFailed {
        url: host.to_owned(),
        reason: e.to_string(),
    })
}

async fn read_limited_body(mut response: reqwest::Response) -> Result<Vec<u8>, PluginHttpError> {
    let url = response.url().to_string();

    if let Some(length) = response.content_length()
        && length > MAX_RESPONSE_BYTES as u64
    {
        return Err(PluginHttpError::ResponseTooLarge {
            limit: MAX_RESPONSE_BYTES,
        });
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
        if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
            return Err(PluginHttpError::ResponseTooLarge {
                limit: MAX_RESPONSE_BYTES,
            });
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

/// The whole `http_get` pipeline, parameterised only by the address filter so that tests can
/// point it at a loopback server without relaxing the production policy.
async fn perform_get(
    request: &HttpRequestDto,
    allowed_hosts: &[String],
    is_blocked: AddressFilter,
) -> Result<HttpResponseDto, PluginHttpError> {
    let (url, host) = parse_url(&request.url)?;

    if !is_host_allowed(&host, allowed_hosts) {
        return Err(PluginHttpError::HostNotAllowed { host });
    }

    let headers = build_headers(&request.headers)?;

    let port = url
        .port_or_known_default()
        .unwrap_or(if url.scheme() == "https" { 443 } else { 80 });
    let addrs = resolve_public_addrs(&host, port, is_blocked).await?;

    let client = build_client(&host, &addrs)?;

    let response = client
        .get(url.clone())
        .headers(headers)
        .send()
        .await
        .map_err(|e| PluginHttpError::RequestFailed {
            url: url.to_string(),
            reason: e.to_string(),
        })?;

    let status = response.status();
    if status.is_redirection() {
        return Err(PluginHttpError::RedirectNotAllowed {
            status: status.as_u16(),
            url: url.to_string(),
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

    let body = read_limited_body(response).await?;

    Ok(HttpResponseDto {
        status: status.as_u16(),
        headers: response_headers,
        body,
    })
}

/// Handle `http_get` — perform an allowlisted, redirect-free GET on behalf of a plugin.
pub(crate) async fn handle_http_get(
    plugin_id: &str,
    request: HttpRequestDto,
    allowed_hosts: &[String],
) -> crate::Result<HttpResponseDto> {
    log::debug!(target: "plugin", "[{plugin_id}]: http_get {}", request.url);

    perform_get(&request, allowed_hosts, is_blocked_ip)
        .await
        .map_err(|err| {
            log::warn!(target: "plugin", "[{plugin_id}]: http_get failed: {err}");
            crate::ErrorKind::from(err).as_error()
        })
}

// ── Extism host function wrappers ──

host_fn!(
pub http_get(user_data: PluginContext; request: Msgpack<HttpRequestDto>) -> HostResult<HttpResponseDto> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.clone();
    let allowed_hosts = ctx.allowed_hosts.clone();
    drop(ctx);

    to_extism_res::<HttpResponseDto>(
        execute_async(handle_http_get(&id, request.0, &allowed_hosts))
    )
});

#[cfg(test)]
mod tests {
    use super::*;

    fn hosts(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).to_string()).collect()
    }

    // ── is_host_allowed ──

    #[test]
    fn should_reject_when_allowed_hosts_is_empty() {
        assert!(!is_host_allowed("example.com", &[]));
    }

    #[test]
    fn should_match_exact_host() {
        assert!(is_host_allowed("example.com", &hosts(&["example.com"])));
        assert!(!is_host_allowed("evil.com", &hosts(&["example.com"])));
    }

    #[test]
    fn should_match_wildcard_subdomain() {
        let allowed = hosts(&["*.example.com"]);
        assert!(is_host_allowed("api.example.com", &allowed));
        assert!(is_host_allowed("a.b.example.com", &allowed));
        // Extism's glob matching requires the dot, so the apex is not covered.
        assert!(!is_host_allowed("example.com", &allowed));
    }

    #[test]
    fn should_not_match_wildcard_suffix_of_another_domain() {
        assert!(!is_host_allowed(
            "api.example.com.evil.com",
            &hosts(&["*.example.com"])
        ));
    }

    #[test]
    fn should_match_full_wildcard() {
        assert!(is_host_allowed("anything.dev", &hosts(&["*"])));
    }

    #[test]
    fn should_match_character_class_like_extism() {
        let allowed = hosts(&["cdn[0-9].example.com"]);
        assert!(is_host_allowed("cdn1.example.com", &allowed));
        assert!(!is_host_allowed("cdnx.example.com", &allowed));
    }

    #[test]
    fn should_fall_back_to_exact_compare_for_invalid_glob() {
        // `[` alone is not a valid glob pattern; Extism compares such entries literally.
        let allowed = hosts(&["exa[mple.com"]);
        assert!(is_host_allowed("exa[mple.com", &allowed));
        assert!(!is_host_allowed("example.com", &allowed));
    }

    #[test]
    fn should_match_any_entry_in_the_list() {
        let allowed = hosts(&["a.example.com", "b.example.com"]);
        assert!(is_host_allowed("b.example.com", &allowed));
        assert!(!is_host_allowed("c.example.com", &allowed));
    }

    // ── is_blocked_ip ──

    #[test]
    fn should_block_loopback() {
        assert!(is_blocked_ip("127.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("127.10.20.30".parse().unwrap()));
        assert!(is_blocked_ip("::1".parse().unwrap()));
    }

    #[test]
    fn should_block_private_ranges() {
        assert!(is_blocked_ip("10.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("172.16.0.1".parse().unwrap()));
        assert!(is_blocked_ip("172.31.255.255".parse().unwrap()));
        assert!(is_blocked_ip("192.168.1.1".parse().unwrap()));
    }

    #[test]
    fn should_block_cloud_metadata_address() {
        assert!(is_blocked_ip("169.254.169.254".parse().unwrap()));
    }

    #[test]
    fn should_block_unspecified_and_broadcast() {
        assert!(is_blocked_ip("0.0.0.0".parse().unwrap()));
        assert!(is_blocked_ip("255.255.255.255".parse().unwrap()));
        assert!(is_blocked_ip("::".parse().unwrap()));
    }

    #[test]
    fn should_block_shared_benchmark_and_reserved_ranges() {
        assert!(is_blocked_ip("100.64.0.1".parse().unwrap()));
        assert!(is_blocked_ip("100.127.255.255".parse().unwrap()));
        assert!(is_blocked_ip("198.18.0.1".parse().unwrap()));
        assert!(is_blocked_ip("192.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("240.0.0.1".parse().unwrap()));
    }

    #[test]
    fn should_block_multicast_and_documentation() {
        assert!(is_blocked_ip("224.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("192.0.2.1".parse().unwrap()));
        assert!(is_blocked_ip("ff02::1".parse().unwrap()));
        assert!(is_blocked_ip("2001:db8::1".parse().unwrap()));
    }

    #[test]
    fn should_block_ipv6_unique_local_and_link_local() {
        assert!(is_blocked_ip("fd00::1".parse().unwrap()));
        assert!(is_blocked_ip("fc00::1".parse().unwrap()));
        assert!(is_blocked_ip("fe80::1".parse().unwrap()));
    }

    #[test]
    fn should_block_ipv4_mapped_private_address() {
        assert!(is_blocked_ip("::ffff:127.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("::ffff:169.254.169.254".parse().unwrap()));
        assert!(is_blocked_ip("::ffff:10.0.0.1".parse().unwrap()));
    }

    #[test]
    fn should_allow_public_addresses() {
        assert!(!is_blocked_ip("1.1.1.1".parse().unwrap()));
        assert!(!is_blocked_ip("8.8.8.8".parse().unwrap()));
        assert!(!is_blocked_ip("104.16.0.1".parse().unwrap()));
        assert!(!is_blocked_ip("2606:4700:4700::1111".parse().unwrap()));
        assert!(!is_blocked_ip("::ffff:8.8.8.8".parse().unwrap()));
    }

    // ── parse_url ──

    #[test]
    fn should_reject_non_http_schemes() {
        for url in ["file:///etc/passwd", "ftp://example.com", "data:,hello"] {
            assert!(
                matches!(
                    parse_url(url),
                    Err(PluginHttpError::UnsupportedScheme { .. }
                        | PluginHttpError::InvalidUrl { .. })
                ),
                "scheme of `{url}` must be rejected"
            );
        }
    }

    #[test]
    fn should_accept_http_and_https() {
        assert!(parse_url("http://example.com/a").is_ok());
        let (_, host) = parse_url("https://example.com:8443/a").unwrap();
        assert_eq!(host, "example.com");
    }

    #[test]
    fn should_reject_malformed_url() {
        assert!(matches!(
            parse_url("not a url"),
            Err(PluginHttpError::InvalidUrl { .. })
        ));
    }

    // ── build_headers ──

    #[test]
    fn should_reject_forbidden_headers() {
        for name in ["Host", "content-length", "Connection", "Transfer-Encoding"] {
            let headers = HashMap::from([(name.to_string(), "x".to_string())]);
            assert!(
                matches!(
                    build_headers(&headers),
                    Err(PluginHttpError::ForbiddenHeader { .. })
                ),
                "header `{name}` must be rejected"
            );
        }
    }

    #[test]
    fn should_accept_ordinary_headers() {
        let headers = HashMap::from([
            ("Accept".to_string(), "application/json".to_string()),
            ("x-api-key".to_string(), "secret".to_string()),
        ]);
        let built = build_headers(&headers).unwrap();
        assert_eq!(built.get("accept").unwrap(), "application/json");
        assert_eq!(built.get("x-api-key").unwrap(), "secret");
    }

    #[test]
    fn should_reject_header_with_invalid_value() {
        let headers = HashMap::from([("x-bad".to_string(), "line\nbreak".to_string())]);
        assert!(matches!(
            build_headers(&headers),
            Err(PluginHttpError::InvalidHeader { .. })
        ));
    }

    // ── handle_http_get: host allowlist enforced before any network access ──

    #[tokio::test]
    async fn should_reject_host_outside_allowed_hosts() {
        let request = HttpRequestDto {
            url: "http://evil.com/".to_string(),
            headers: HashMap::new(),
        };
        let err = handle_http_get("test", request, &hosts(&["example.com"]))
            .await
            .expect_err("request to a host outside allowed_hosts must fail");
        assert!(err.to_string().contains("allowed_hosts"), "{err}");
    }

    #[tokio::test]
    async fn should_reject_loopback_even_when_host_is_allowed() {
        let request = HttpRequestDto {
            url: "http://127.0.0.1:9/".to_string(),
            headers: HashMap::new(),
        };
        let err = handle_http_get("test", request, &hosts(&["127.0.0.1"]))
            .await
            .expect_err("loopback must be blocked even if explicitly allowlisted");
        assert!(err.to_string().contains("non-public address"), "{err}");
    }

    #[tokio::test]
    async fn should_reject_metadata_address_even_when_host_is_allowed() {
        let request = HttpRequestDto {
            url: "http://169.254.169.254/latest/meta-data/".to_string(),
            headers: HashMap::new(),
        };
        let err = handle_http_get("test", request, &hosts(&["*"]))
            .await
            .expect_err("cloud metadata address must be blocked");
        assert!(err.to_string().contains("non-public address"), "{err}");
    }

    // ── Against a real local HTTP server ──
    //
    // These use a permissive address filter, because the test server necessarily lives on
    // loopback, which the production filter blocks (as the tests above assert).

    fn allow_any_address(_ip: IpAddr) -> bool {
        false
    }

    /// Serve exactly one request with the given raw response, then close.
    async fn serve_once(raw_response: &'static str) -> SocketAddr {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test server should bind");
        let addr = listener.local_addr().expect("test server should have addr");

        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};

            let (mut stream, _) = listener.accept().await.expect("should accept connection");

            // Drain the request headers so the client is not left writing into a closed socket.
            let mut buf = [0_u8; 1024];
            let _ = stream.read(&mut buf).await;

            let _ = stream.write_all(raw_response.as_bytes()).await;
            let _ = stream.flush().await;
        });

        addr
    }

    async fn get_from(
        addr: SocketAddr,
        is_blocked: AddressFilter,
    ) -> Result<HttpResponseDto, PluginHttpError> {
        let request = HttpRequestDto {
            url: format!("http://127.0.0.1:{}/pack.toml", addr.port()),
            headers: HashMap::new(),
        };
        perform_get(&request, &hosts(&["127.0.0.1"]), is_blocked).await
    }

    #[tokio::test]
    async fn should_fail_on_redirect_to_another_host_instead_of_following_it() {
        let addr = serve_once(
            "HTTP/1.1 302 Found\r\nLocation: http://169.254.169.254/latest/meta-data/\r\nContent-Length: 0\r\n\r\n",
        )
        .await;

        let err = get_from(addr, allow_any_address)
            .await
            .expect_err("a redirect must be an error, not a hop");

        assert!(
            matches!(err, PluginHttpError::RedirectNotAllowed { status: 302, .. }),
            "expected RedirectNotAllowed, got: {err}"
        );
    }

    #[tokio::test]
    async fn should_return_body_and_headers_for_a_plain_response() {
        let addr = serve_once(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello",
        )
        .await;

        let response = get_from(addr, allow_any_address)
            .await
            .expect("plain response should be returned");

        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"hello");
        assert_eq!(
            response.headers.get("content-type").map(String::as_str),
            Some("text/plain")
        );
    }

    #[tokio::test]
    async fn should_reject_response_larger_than_the_limit() {
        // Claim a body far over the limit; the Content-Length check must fire before any
        // of it is read.
        let addr = serve_once("HTTP/1.1 200 OK\r\nContent-Length: 1073741824\r\n\r\nstart").await;

        let err = get_from(addr, allow_any_address)
            .await
            .expect_err("oversized body must be refused");

        assert!(
            matches!(err, PluginHttpError::ResponseTooLarge { .. }),
            "expected ResponseTooLarge, got: {err}"
        );
    }

    #[tokio::test]
    async fn should_still_block_loopback_with_the_production_filter() {
        let addr = serve_once("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").await;

        let err = get_from(addr, is_blocked_ip)
            .await
            .expect_err("the production filter must block loopback");

        assert!(
            matches!(err, PluginHttpError::BlockedAddress { .. }),
            "expected BlockedAddress, got: {err}"
        );
    }
}
