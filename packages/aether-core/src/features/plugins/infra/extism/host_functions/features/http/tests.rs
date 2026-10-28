use std::net::Ipv4Addr;

use super::super::test_server::{TestServer, ok, redirect, status};
use reqwest::{StatusCode, dns::Resolve, header::HeaderValue};

use super::policy::{redirect_target, resolve_public_addrs};
use super::*;

/// Two loopback addresses: two different hosts for `allowed_hosts`, the address filter and
/// the origin.
const FIRST: &str = "127.0.0.1";
const SECOND: &str = "127.0.0.2";

fn limits() -> PluginSandboxLimits {
    PluginSandboxLimits::default()
}

fn hosts(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_string()).collect()
}

// ── is_host_allowed ──

#[test]
fn should_reject_when_allowed_hosts_is_empty() {
    assert!(!is_host_allowed("example.com", &[]));
}

#[test]
fn should_match_when_the_host_is_exact() {
    assert!(is_host_allowed("example.com", &hosts(&["example.com"])));
    assert!(!is_host_allowed("evil.com", &hosts(&["example.com"])));
}

#[test]
fn should_match_when_the_host_is_a_wildcard_subdomain() {
    let allowed = hosts(&["*.example.com"]);
    assert!(is_host_allowed("api.example.com", &allowed));
    assert!(is_host_allowed("a.b.example.com", &allowed));
    // Extism's glob matching requires the dot, so the apex is not covered.
    assert!(!is_host_allowed("example.com", &allowed));
}

#[test]
fn should_not_match_when_the_host_is_a_suffix_of_another_domain() {
    assert!(!is_host_allowed(
        "api.example.com.evil.com",
        &hosts(&["*.example.com"])
    ));
}

#[test]
fn should_match_when_the_entry_is_a_full_wildcard() {
    assert!(is_host_allowed("anything.dev", &hosts(&["*"])));
}

#[test]
fn should_match_when_the_entry_is_a_character_class_like_extism() {
    let allowed = hosts(&["cdn[0-9].example.com"]);
    assert!(is_host_allowed("cdn1.example.com", &allowed));
    assert!(!is_host_allowed("cdnx.example.com", &allowed));
}

#[test]
fn should_fall_back_to_exact_compare_when_the_glob_is_invalid() {
    // `[` alone is not a valid glob pattern; Extism compares such entries literally.
    let allowed = hosts(&["exa[mple.com"]);
    assert!(is_host_allowed("exa[mple.com", &allowed));
    assert!(!is_host_allowed("example.com", &allowed));
}

#[test]
fn should_match_when_any_entry_in_the_list_matches() {
    let allowed = hosts(&["a.example.com", "b.example.com"]);
    assert!(is_host_allowed("b.example.com", &allowed));
    assert!(!is_host_allowed("c.example.com", &allowed));
}

// ── is_blocked_ip ──

#[test]
fn should_block_when_the_address_is_loopback() {
    assert!(is_blocked_ip("127.0.0.1".parse().unwrap()));
    assert!(is_blocked_ip("127.10.20.30".parse().unwrap()));
    assert!(is_blocked_ip("::1".parse().unwrap()));
}

#[test]
fn should_block_when_the_address_is_in_a_private_range() {
    assert!(is_blocked_ip("10.0.0.1".parse().unwrap()));
    assert!(is_blocked_ip("172.16.0.1".parse().unwrap()));
    assert!(is_blocked_ip("172.31.255.255".parse().unwrap()));
    assert!(is_blocked_ip("192.168.1.1".parse().unwrap()));
}

#[test]
fn should_block_when_the_address_is_the_cloud_metadata_address() {
    assert!(is_blocked_ip("169.254.169.254".parse().unwrap()));
}

#[test]
fn should_block_when_the_address_is_unspecified_or_broadcast() {
    assert!(is_blocked_ip("0.0.0.0".parse().unwrap()));
    assert!(is_blocked_ip("255.255.255.255".parse().unwrap()));
    assert!(is_blocked_ip("::".parse().unwrap()));
}

#[test]
fn should_block_when_the_address_is_shared_benchmark_or_reserved() {
    assert!(is_blocked_ip("100.64.0.1".parse().unwrap()));
    assert!(is_blocked_ip("100.127.255.255".parse().unwrap()));
    assert!(is_blocked_ip("198.18.0.1".parse().unwrap()));
    assert!(is_blocked_ip("192.0.0.1".parse().unwrap()));
    assert!(is_blocked_ip("240.0.0.1".parse().unwrap()));
}

#[test]
fn should_block_when_the_address_is_multicast_or_documentation() {
    assert!(is_blocked_ip("224.0.0.1".parse().unwrap()));
    assert!(is_blocked_ip("192.0.2.1".parse().unwrap()));
    assert!(is_blocked_ip("ff02::1".parse().unwrap()));
    assert!(is_blocked_ip("2001:db8::1".parse().unwrap()));
}

#[test]
fn should_block_when_an_ipv4_mapped_address_is_private() {
    assert!(is_blocked_ip("::ffff:127.0.0.1".parse().unwrap()));
    assert!(is_blocked_ip("::ffff:169.254.169.254".parse().unwrap()));
    assert!(is_blocked_ip("::ffff:10.0.0.1".parse().unwrap()));
}

#[test]
fn should_allow_when_the_address_is_public() {
    assert!(!is_blocked_ip("1.1.1.1".parse().unwrap()));
    assert!(!is_blocked_ip("8.8.8.8".parse().unwrap()));
    assert!(!is_blocked_ip("104.16.0.1".parse().unwrap()));
    assert!(!is_blocked_ip("2606:4700:4700::1111".parse().unwrap()));
    assert!(!is_blocked_ip("::ffff:8.8.8.8".parse().unwrap()));
}

// ── parse_url ──

#[test]
fn should_reject_when_the_scheme_is_not_http() {
    for url in ["file:///etc/passwd", "ftp://example.com", "data:,hello"] {
        assert!(
            matches!(
                parse_url(url),
                Err(PluginHttpError::UnsupportedScheme { .. } | PluginHttpError::InvalidUrl { .. })
            ),
            "scheme of `{url}` must be rejected"
        );
    }
}

#[test]
fn should_accept_when_the_scheme_is_http_or_https() {
    assert!(parse_url("http://example.com/a").is_ok());
    let (_, host) = parse_url("https://example.com:8443/a").unwrap();
    assert_eq!(host, "example.com");
}

#[test]
fn should_reject_when_the_url_is_malformed() {
    assert!(matches!(
        parse_url("not a url"),
        Err(PluginHttpError::InvalidUrl { .. })
    ));
}

// ── build_headers ──

#[test]
fn should_reject_when_a_header_is_forbidden() {
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
fn should_accept_when_headers_are_ordinary() {
    let headers = HashMap::from([
        ("Accept".to_string(), "application/json".to_string()),
        ("x-api-key".to_string(), "secret".to_string()),
    ]);
    let built = build_headers(&headers).unwrap();
    assert_eq!(built.get("accept").unwrap(), "application/json");
    assert_eq!(built.get("x-api-key").unwrap(), "secret");
}

#[test]
fn should_reject_when_a_header_has_an_invalid_value() {
    let headers = HashMap::from([("x-bad".to_string(), "line\nbreak".to_string())]);
    assert!(matches!(
        build_headers(&headers),
        Err(PluginHttpError::InvalidHeader { .. })
    ));
}

// ── handle_http_get: host allowlist enforced before any network access ──

#[tokio::test]
async fn should_reject_when_the_host_is_outside_allowed_hosts() {
    let request = HttpRequestDto {
        url: "http://evil.com/".to_string(),
        headers: HashMap::new(),
    };
    let err = handle_http_get("test", request, &hosts(&["example.com"]), limits())
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
    let err = handle_http_get("test", request, &hosts(&["127.0.0.1"]), limits())
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
    let err = handle_http_get("test", request, &hosts(&["*"]), limits())
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
    perform_get(
        &request,
        &hosts(&["127.0.0.1"]),
        is_blocked,
        &ProxySource::Direct,
        limits(),
    )
    .await
}

#[tokio::test]
async fn should_return_body_and_headers_when_the_response_is_plain() {
    let addr =
        serve_once("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello")
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
async fn should_reject_the_response_when_it_is_larger_than_the_limit() {
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

// ── http_post_json ──

fn post_request(url: String, body: &str) -> HttpPostJsonRequestDto {
    HttpPostJsonRequestDto {
        url,
        headers: HashMap::new(),
        body: body.to_string(),
    }
}

/// Like `serve_once`, but hands the raw request back so a test can inspect it.
async fn serve_capturing(
    raw_response: &'static str,
) -> (SocketAddr, tokio::sync::oneshot::Receiver<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test server should bind");
    let addr = listener.local_addr().expect("test server should have addr");
    let (tx, rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let (mut stream, _) = listener.accept().await.expect("should accept connection");
        // Headers and body may arrive in separate segments: read the whole request, or an
        // early close with unread data would reset the connection under the client.
        let mut received = Vec::new();
        let mut buf = [0_u8; 1024];
        loop {
            let text = String::from_utf8_lossy(&received).to_ascii_lowercase();
            if let Some((head, body)) = text.split_once("\r\n\r\n") {
                let expected = head
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length: "))
                    .and_then(|value| value.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                if body.len() >= expected {
                    break;
                }
            }
            match stream.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(n) => received.extend_from_slice(&buf[..n]),
            }
        }
        let _ = tx.send(String::from_utf8_lossy(&received).into_owned());

        let _ = stream.write_all(raw_response.as_bytes()).await;
        let _ = stream.flush().await;
    });

    (addr, rx)
}

#[tokio::test]
async fn should_reject_a_post_when_the_host_is_outside_allowed_hosts() {
    let request = post_request("http://evil.com/api".to_string(), "{}");
    let err = handle_http_post_json("test", request, &hosts(&["example.com"]), limits())
        .await
        .expect_err("POST to a host outside allowed_hosts must fail");
    assert!(err.to_string().contains("allowed_hosts"), "{err}");
}

#[tokio::test]
async fn should_reject_post_to_non_public_address_even_when_allowlisted() {
    for url in ["http://127.0.0.1:9/", "http://169.254.169.254/latest/"] {
        let request = post_request(url.to_string(), "{}");
        let err = handle_http_post_json("test", request, &hosts(&["*"]), limits())
            .await
            .expect_err("non-public address must be blocked");
        assert!(err.to_string().contains("non-public address"), "{err}");
    }
}

#[tokio::test]
async fn should_fail_instead_of_following_when_a_post_is_redirected() {
    // POST follows nothing, even to a host that is allowlisted.
    let target = TestServer::start(SECOND, |_| ok(b"{}")).await;
    for code in [301, 302, 303, 307, 308] {
        let location = target.url("/moved");
        let origin = TestServer::start(FIRST, move |_| redirect(code, &location)).await;
        let request = post_request(origin.url("/x"), "{}");

        let err = perform_post_json(
            &request,
            &hosts(&[FIRST, SECOND]),
            allow_any_address,
            &ProxySource::Direct,
            limits(),
        )
        .await
        .expect_err("a redirect must be an error, not a hop");

        assert!(
            matches!(err, PluginHttpError::RedirectNotAllowed { status, .. } if status == code),
            "{code}: expected RedirectNotAllowed, got: {err}"
        );
    }
    assert_eq!(
        target.connections(),
        0,
        "the redirect target must not be contacted"
    );
}

#[tokio::test]
async fn should_send_json_with_a_fixed_content_type_and_drop_other_headers_when_posting() {
    let (addr, captured) = serve_capturing("HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok").await;
    let request = HttpPostJsonRequestDto {
        url: format!("http://127.0.0.1:{}/v1/search", addr.port()),
        headers: HashMap::from([
            ("X-Api-Key".to_string(), "k".to_string()),
            ("Content-Type".to_string(), "text/evil".to_string()),
            ("Host".to_string(), "other.example".to_string()),
            ("Cookie".to_string(), "a=b".to_string()),
            ("X-Forwarded-For".to_string(), "10.0.0.1".to_string()),
        ]),
        body: r#"{"q":"sodium"}"#.to_string(),
    };

    let response = perform_post_json(
        &request,
        &hosts(&["127.0.0.1"]),
        allow_any_address,
        &ProxySource::Direct,
        limits(),
    )
    .await
    .expect("POST should succeed");
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"ok");

    let raw = captured
        .await
        .expect("server should capture the request")
        .to_ascii_lowercase();
    assert!(raw.starts_with("post /v1/search "), "{raw}");
    assert!(raw.contains("content-type: application/json"), "{raw}");
    assert!(raw.contains("x-api-key: k"), "{raw}");
    assert!(raw.ends_with(r#"{"q":"sodium"}"#), "{raw}");
    for dropped in ["text/evil", "other.example", "cookie", "x-forwarded-for"] {
        assert!(!raw.contains(dropped), "`{dropped}` must be dropped: {raw}");
    }
}

#[test]
fn should_keep_only_allowlisted_headers_when_posting() {
    let headers = HashMap::from([
        ("Accept".to_string(), "application/json".to_string()),
        ("Authorization".to_string(), "Bearer t".to_string()),
        ("Content-Type".to_string(), "text/plain".to_string()),
        ("Host".to_string(), "x".to_string()),
        ("Cookie".to_string(), "a=b".to_string()),
    ]);
    let built = filter_post_headers(&headers).unwrap();
    assert_eq!(built.len(), 2);
    assert!(built.contains_key("accept"));
    assert!(built.contains_key("authorization"));
}

#[test]
fn should_reject_a_post_when_the_body_is_not_json_or_is_oversized() {
    assert!(validate_json_body(r#"{"a":[1,2,3]}"#, limits().max_request_body_bytes).is_ok());
    assert!(matches!(
        validate_json_body("{not json", limits().max_request_body_bytes),
        Err(PluginHttpError::InvalidJsonBody { .. })
    ));
    assert!(matches!(
        validate_json_body(
            &" ".repeat(limits().max_request_body_bytes + 1),
            limits().max_request_body_bytes
        ),
        Err(PluginHttpError::RequestBodyTooLarge { .. })
    ));
}

#[tokio::test]
async fn should_resolve_a_public_ipv6_literal_when_the_host_keeps_its_brackets() {
    let (_, host) = parse_url("https://[2606:4700:4700::1111]/").unwrap();

    let addrs = resolve_public_addrs(&host, 443, system_lookup, is_blocked_ip)
        .await
        .expect("a public IPv6 literal must resolve and pass the filter");

    assert_eq!(addrs.len(), 1);
    assert_eq!(
        addrs[0].ip(),
        "2606:4700:4700::1111".parse::<IpAddr>().unwrap()
    );
}

#[tokio::test]
async fn should_refuse_a_non_public_ipv6_literal_when_the_host_keeps_its_brackets() {
    for url in [
        "http://[::1]/",
        "http://[fc00::1]/",
        "http://[fd12:3456::1]/",
    ] {
        let (_, host) = parse_url(url).unwrap();

        let err = resolve_public_addrs(&host, 80, system_lookup, is_blocked_ip)
            .await
            .expect_err("a non-public IPv6 literal must be refused");

        assert!(
            matches!(err, PluginHttpError::BlockedAddress { .. }),
            "{url}: expected BlockedAddress, got: {err}"
        );
    }
}

#[tokio::test]
async fn should_still_block_loopback_when_the_production_filter_is_used() {
    let addr = serve_once("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").await;

    let err = get_from(addr, is_blocked_ip)
        .await
        .expect_err("the production filter must block loopback");

    assert!(
        matches!(err, PluginHttpError::BlockedAddress { .. }),
        "expected BlockedAddress, got: {err}"
    );
}

// ── Redirects: followed for GET, every hop vetted like the first request ──

async fn get_url(
    url: String,
    headers: &[(&str, &str)],
    allowed: &[&str],
    is_blocked: AddressFilter,
) -> Result<HttpResponseDto, PluginHttpError> {
    let request = HttpRequestDto {
        url,
        headers: headers
            .iter()
            .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
            .collect(),
    };
    perform_get(
        &request,
        &hosts(allowed),
        is_blocked,
        &ProxySource::Direct,
        limits(),
    )
    .await
}

/// The production filter with one exception, the first test server, so that a test can
/// start on loopback and still have its *next* hop judged by the real policy.
fn production_filter_except_first(ip: IpAddr) -> bool {
    ip != IpAddr::V4(Ipv4Addr::LOCALHOST) && is_blocked_ip(ip)
}

/// `/hops/N` redirects to `/hops/N-1`; `/hops/0` answers.
fn chain(path: &str) -> Vec<u8> {
    let n: usize = path
        .strip_prefix("/hops/")
        .and_then(|n| n.parse().ok())
        .expect("chain path");
    if n == 0 {
        ok(b"end")
    } else {
        redirect(302, &format!("/hops/{}", n - 1))
    }
}

#[tokio::test]
async fn should_follow_a_redirect_when_it_leads_to_an_allowlisted_host() {
    let target = TestServer::start(SECOND, |_| ok(b"final")).await;
    let location = target.url("/file.jar");
    let origin = TestServer::start(FIRST, move |_| redirect(302, &location)).await;

    let response = get_url(
        origin.url("/latest"),
        &[],
        &[FIRST, SECOND],
        allow_any_address,
    )
    .await
    .expect("an allowlisted hop should be followed");

    assert_eq!(response.status, 200);
    assert_eq!(
        response.body, b"final",
        "the redirect's own body is not the result"
    );
    let requests = target.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].starts_with("get /file.jar "), "{}", requests[0]);
}

#[tokio::test]
async fn should_follow_every_redirect_status_when_the_method_is_get() {
    for code in [301, 302, 303, 307, 308] {
        let server = TestServer::start(FIRST, move |path| match path {
            "/start" => redirect(code, "/end"),
            _ => ok(b"end"),
        })
        .await;

        let response = get_url(server.url("/start"), &[], &[FIRST], allow_any_address)
            .await
            .unwrap_or_else(|err| panic!("{code} should be followed: {err}"));
        assert_eq!(response.body, b"end", "{code}");
    }
}

#[tokio::test]
async fn should_refuse_a_redirect_without_connecting_when_it_leads_outside_allowed_hosts() {
    let target = TestServer::start(SECOND, |_| ok(b"secret")).await;
    let location = target.url("/");
    let origin = TestServer::start(FIRST, move |_| redirect(302, &location)).await;

    let err = get_url(origin.url("/"), &[], &[FIRST], allow_any_address)
        .await
        .expect_err("a hop to a host outside allowed_hosts must be refused");

    assert!(
        matches!(err, PluginHttpError::HostNotAllowed { ref host } if host == SECOND),
        "expected HostNotAllowed, got: {err}"
    );
    assert_eq!(
        target.connections(),
        0,
        "the refused host must not be contacted"
    );
}

#[tokio::test]
async fn should_refuse_a_redirect_to_a_non_public_address_when_the_production_filter_is_used() {
    // A real server sits at the loopback target, so a connection would show in its counter.
    let target = TestServer::start(SECOND, |_| ok(b"internal")).await;
    let location = target.url("/");
    let origin = TestServer::start(FIRST, move |path| match path {
        "/metadata" => redirect(302, "http://169.254.169.254/latest/meta-data/"),
        "/private" => redirect(302, "http://10.0.0.1/"),
        _ => redirect(302, &location),
    })
    .await;

    for path in ["/loopback", "/metadata", "/private"] {
        let err = get_url(
            origin.url(path),
            &[],
            &["*"],
            production_filter_except_first,
        )
        .await
        .expect_err("a hop to a non-public address must be refused");

        assert!(
            matches!(err, PluginHttpError::BlockedAddress { .. }),
            "{path}: expected BlockedAddress, got: {err}"
        );
    }
    assert_eq!(
        target.connections(),
        0,
        "the blocked host must not be contacted"
    );
}

#[tokio::test]
async fn should_refuse_the_sixth_redirect_when_five_were_followed() {
    let max_redirects = limits().max_redirects;
    let server = TestServer::start(FIRST, chain).await;
    let response = get_url(
        server.url(&format!("/hops/{max_redirects}")),
        &[],
        &[FIRST],
        allow_any_address,
    )
    .await
    .expect("five redirects are within the limit");
    assert_eq!(response.body, b"end");
    assert_eq!(server.connections(), max_redirects + 1);

    let server = TestServer::start(FIRST, chain).await;
    let err = get_url(
        server.url(&format!("/hops/{}", max_redirects + 1)),
        &[],
        &[FIRST],
        allow_any_address,
    )
    .await
    .expect_err("a sixth redirect must be an error");
    assert!(
        matches!(
            err,
            PluginHttpError::TooManyRedirects { limit } if limit == max_redirects
        ),
        "expected TooManyRedirects, got: {err}"
    );
    assert_eq!(
        server.connections(),
        max_redirects + 1,
        "the sixth Location must not be requested"
    );
}

#[tokio::test]
async fn should_resolve_a_relative_location_against_the_current_url_when_redirected() {
    let server = TestServer::start(FIRST, |path| match path {
        "/dir/start" => redirect(302, "next?x=1"),
        "/dir/next?x=1" => redirect(307, "../final"),
        other => ok(other.as_bytes()),
    })
    .await;

    let response = get_url(server.url("/dir/start"), &[], &[FIRST], allow_any_address)
        .await
        .expect("relative Locations should be followed");

    assert_eq!(response.body, b"/final");
}

#[tokio::test]
async fn should_fail_when_a_redirect_has_no_location() {
    let server = TestServer::start(FIRST, |_| status(302)).await;

    let err = get_url(server.url("/"), &[], &[FIRST], allow_any_address)
        .await
        .expect_err("a redirect with nowhere to go must be an error");

    assert!(
        matches!(err, PluginHttpError::InvalidRedirect { status: 302, .. }),
        "expected InvalidRedirect, got: {err}"
    );
}

#[tokio::test]
async fn should_fail_when_a_3xx_is_not_a_redirect() {
    let server = TestServer::start(FIRST, |_| status(304)).await;

    let err = get_url(server.url("/"), &[], &[FIRST], allow_any_address)
        .await
        .expect_err("a 304 is not followed and is not a usable response");

    assert!(
        matches!(err, PluginHttpError::RedirectNotAllowed { status: 304, .. }),
        "expected RedirectNotAllowed, got: {err}"
    );
}

#[tokio::test]
async fn should_not_carry_plugin_headers_when_a_redirect_leads_to_another_host() {
    // first: /start → /same (same origin) → second: /other → first: /back
    let first_url = Arc::new(std::sync::OnceLock::<String>::new());
    let back = first_url.clone();
    let second = TestServer::start(SECOND, move |_| {
        redirect(
            302,
            &format!("{}/back", back.get().expect("first server url")),
        )
    })
    .await;
    let other = second.url("/other");
    let first = TestServer::start(FIRST, move |path| match path {
        "/start" => redirect(302, "/same"),
        "/same" => redirect(302, &other),
        _ => ok(b"done"),
    })
    .await;
    first_url
        .set(first.url(""))
        .expect("first server url is set once");

    let response = get_url(
        first.url("/start"),
        &[("X-Api-Key", "secret")],
        &[FIRST, SECOND],
        allow_any_address,
    )
    .await
    .expect("the chain should complete");
    assert_eq!(response.body, b"done");

    let first_requests = first.requests();
    let second_requests = second.requests();
    assert_eq!(first_requests.len(), 3, "{first_requests:?}");
    assert_eq!(second_requests.len(), 1, "{second_requests:?}");

    let has_key = |raw: &String| raw.contains("x-api-key: secret");
    assert!(
        has_key(&first_requests[0]),
        "the first request carries the key"
    );
    assert!(
        has_key(&first_requests[1]),
        "a same-origin hop keeps the key"
    );
    assert!(
        !has_key(&second_requests[0]),
        "the key must not reach another host: {}",
        second_requests[0]
    );
    assert!(
        !has_key(&first_requests[2]),
        "once dropped, the key stays dropped: {}",
        first_requests[2]
    );
}

#[test]
fn should_refuse_a_redirect_when_it_goes_from_https_to_http() {
    let from = Url::parse("https://github.com/owner/repo/releases/latest/download/a.jar").unwrap();
    let downgrade = HeaderValue::from_static("http://release-assets.githubusercontent.com/a");

    assert!(
        matches!(
            redirect_target(&from, StatusCode::FOUND, Some(&downgrade)),
            Err(PluginHttpError::SchemeDowngrade { .. })
        ),
        "https → http must be refused"
    );

    // A relative Location keeps the scheme; staying on https or upgrading is fine.
    let relative = HeaderValue::from_static("/owner/repo/releases/download/v1/a.jar");
    assert_eq!(
        redirect_target(&from, StatusCode::FOUND, Some(&relative))
            .unwrap()
            .as_str(),
        "https://github.com/owner/repo/releases/download/v1/a.jar"
    );
    let cdn = HeaderValue::from_static("https://release-assets.githubusercontent.com/a?sig=x");
    assert!(redirect_target(&from, StatusCode::FOUND, Some(&cdn)).is_ok());
    let plain = Url::parse("http://example.com/").unwrap();
    let upgrade = HeaderValue::from_static("https://example.com/");
    assert!(redirect_target(&plain, StatusCode::MOVED_PERMANENTLY, Some(&upgrade)).is_ok());
}

#[test]
fn should_refuse_a_redirect_when_the_location_is_not_a_url() {
    let from = Url::parse("https://example.com/a").unwrap();
    for location in [
        HeaderValue::from_bytes(b"\xff\xfe").unwrap(),
        HeaderValue::from_static("http://[not a host/"),
    ] {
        assert!(
            matches!(
                redirect_target(&from, StatusCode::FOUND, Some(&location)),
                Err(PluginHttpError::InvalidRedirect { .. })
            ),
            "{location:?}"
        );
    }
    assert!(matches!(
        redirect_target(&from, StatusCode::FOUND, None),
        Err(PluginHttpError::InvalidRedirect { .. })
    ));
}

#[tokio::test]
async fn should_not_resolve_a_name_when_it_was_not_vetted() {
    let resolver = PinnedResolver::default();
    assert!(
        resolver
            .resolve("example.com".parse().unwrap())
            .await
            .is_err(),
        "an unvetted name must fail to resolve"
    );

    let vetted: SocketAddr = "93.184.215.14:443".parse().unwrap();
    resolver.pin("example.com".to_string(), vec![vetted]);
    let addrs: Vec<SocketAddr> = resolver
        .resolve("example.com".parse().unwrap())
        .await
        .expect("a vetted name resolves")
        .collect();
    assert_eq!(
        addrs,
        vec![vetted],
        "only the vetted addresses are returned"
    );
}

// ── Behind a proxy ──
//
// The test server plays an HTTP proxy: a request through it arrives in absolute form, so the
// handler sees the whole target URL as its "path". The targets live under `.invalid`, which
// never resolves (RFC 2606): a success proves the name was not resolved locally.

async fn get_via(
    proxy: &TestServer,
    url: &str,
    headers: &[(&str, &str)],
    allowed: &[&str],
) -> Result<HttpResponseDto, PluginHttpError> {
    let request = HttpRequestDto {
        url: url.to_string(),
        headers: headers
            .iter()
            .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
            .collect(),
    };
    // The production filter throughout: the proxy sits on loopback and must still be
    // reachable, because the user configured it, not the plugin.
    perform_get(
        &request,
        &hosts(allowed),
        is_blocked_ip,
        &ProxySource::Fixed(proxy.url("")),
        limits(),
    )
    .await
}

#[tokio::test]
async fn should_leave_name_resolution_to_the_proxy_when_one_is_configured() {
    let proxy = TestServer::start(FIRST, |target| ok(target.as_bytes())).await;

    let response = get_via(
        &proxy,
        "http://cdn.plugin.invalid/file.jar",
        &[],
        &["cdn.plugin.invalid"],
    )
    .await
    .expect("a proxied request for an allowlisted name should go through");

    assert_eq!(response.body, b"http://cdn.plugin.invalid/file.jar");
}

#[tokio::test]
async fn should_reach_a_proxy_when_it_is_given_by_name() {
    let proxy = TestServer::start(FIRST, |_| ok(b"via named proxy")).await;
    let port = proxy.url("").rsplit(':').next().unwrap().to_owned();
    let request = HttpRequestDto {
        url: "http://cdn.plugin.invalid/".to_string(),
        headers: HashMap::new(),
    };

    // `localhost` has to resolve, and the client's resolver only answers for pinned names.
    let response = perform_get(
        &request,
        &hosts(&["cdn.plugin.invalid"]),
        is_blocked_ip,
        &ProxySource::Fixed(format!("http://localhost:{port}")),
        limits(),
    )
    .await
    .expect("a proxy given by name should be reachable");

    assert_eq!(response.body, b"via named proxy");
}

#[tokio::test]
async fn should_check_allowed_hosts_on_every_hop_when_going_through_a_proxy() {
    let proxy = TestServer::start(FIRST, |target| match target {
        "http://a.plugin.invalid/start" => redirect(302, "http://b.plugin.invalid/x"),
        _ => ok(b"must not be fetched"),
    })
    .await;

    let err = get_via(
        &proxy,
        "http://a.plugin.invalid/start",
        &[],
        &["a.plugin.invalid"],
    )
    .await
    .expect_err("a hop outside allowed_hosts must be refused behind a proxy too");

    assert!(
        matches!(err, PluginHttpError::HostNotAllowed { ref host } if host == "b.plugin.invalid"),
        "expected HostNotAllowed, got: {err}"
    );
    assert_eq!(
        proxy.requests().len(),
        1,
        "the refused hop must not reach the proxy"
    );
}

#[tokio::test]
async fn should_still_filter_ip_literal_hosts_when_going_through_a_proxy() {
    let proxy = TestServer::start(FIRST, |target| match target {
        "http://a.plugin.invalid/start" => {
            redirect(302, "http://169.254.169.254/latest/meta-data/")
        }
        _ => ok(b"must not be fetched"),
    })
    .await;

    let err = get_via(&proxy, "http://10.0.0.1/", &[], &["*"])
        .await
        .expect_err("a private IP literal must be refused behind a proxy");
    assert!(
        matches!(err, PluginHttpError::BlockedAddress { .. }),
        "expected BlockedAddress, got: {err}"
    );
    assert_eq!(proxy.connections(), 0);

    let err = get_via(&proxy, "http://a.plugin.invalid/start", &[], &["*"])
        .await
        .expect_err("a hop to the metadata address must be refused behind a proxy");
    assert!(
        matches!(err, PluginHttpError::BlockedAddress { .. }),
        "expected BlockedAddress, got: {err}"
    );
    assert_eq!(
        proxy.requests().len(),
        1,
        "the refused hop must not reach the proxy"
    );
}

#[tokio::test]
async fn should_drop_plugin_headers_across_origins_when_going_through_a_proxy() {
    let proxy = TestServer::start(FIRST, |target| match target {
        "http://a.plugin.invalid/start" => redirect(302, "http://b.plugin.invalid/other"),
        _ => ok(b"done"),
    })
    .await;

    let response = get_via(
        &proxy,
        "http://a.plugin.invalid/start",
        &[("X-Api-Key", "secret")],
        &["a.plugin.invalid", "b.plugin.invalid"],
    )
    .await
    .expect("the chain should complete");
    assert_eq!(response.body, b"done");

    let requests = proxy.requests();
    assert_eq!(requests.len(), 2, "{requests:?}");
    assert!(requests[0].contains("x-api-key: secret"), "{}", requests[0]);
    assert!(
        !requests[1].contains("x-api-key"),
        "the key must not reach another origin: {}",
        requests[1]
    );
}

// ── Clash fake-ip ──
//
// With fake-ip, the DNS Clash serves answers every name with an address from
// `198.18.0.0/15` and routes by the name later. Without a proxy configured (TUN mode), such
// a name must be refused; through the proxy, the fake address must not matter at all.

const FAKE_IP: [u8; 4] = [198, 18, 0, 7];

fn fake_ip_dns(_host: String, port: u16) -> BoxFuture<'static, std::io::Result<Vec<SocketAddr>>> {
    Box::pin(async move { Ok(vec![SocketAddr::from((FAKE_IP, port))]) })
}

/// GET `url` through a vetted client whose DNS hands out fake-ip addresses, with the
/// production address filter.
async fn get_with_fake_ip_dns(url: &str, proxy: &ProxySource) -> Result<Vec<u8>, PluginHttpError> {
    let url = Url::parse(url).unwrap();
    let client = vetted_client(
        &hosts(&["github.com"]),
        fake_ip_dns,
        is_blocked_ip,
        proxy,
        REQUEST_TIMEOUT,
        None,
        limits().max_redirects,
    )?;
    let response = client
        .get(url.clone())
        .send()
        .await
        .map_err(|err| send_error(&url, err))?;
    Ok(response.bytes().await.expect("test body").to_vec())
}

#[tokio::test]
async fn should_refuse_a_fake_ip_answer_when_there_is_no_proxy() {
    let err = get_with_fake_ip_dns("http://github.com/x.jar", &ProxySource::Direct)
        .await
        .expect_err("a name resolving into 198.18.0.0/15 must be refused when connecting directly");

    assert!(
        matches!(
            err,
            PluginHttpError::BlockedAddress { ref host, ip }
                if host == "github.com" && ip == IpAddr::from(FAKE_IP)
        ),
        "expected BlockedAddress for the fake-ip address, got: {err}"
    );
}

#[tokio::test]
async fn should_not_care_about_fake_ip_answers_when_going_through_a_proxy() {
    let proxy = TestServer::start(FIRST, |target| ok(target.as_bytes())).await;

    let body = get_with_fake_ip_dns(
        "http://github.com/x.jar",
        &ProxySource::Fixed(proxy.url("")),
    )
    .await
    .expect("through the proxy, the name is the proxy's to resolve");

    assert_eq!(body, b"http://github.com/x.jar");
    assert_eq!(proxy.requests().len(), 1);
}

// ── DNS rebinding ──
//
// A name that resolves to a public address while it is checked and to a private one right
// after. The connection must go only where the check let it, and the private answer must be
// refused whenever the name is asked about again.

const REBINDING_HOST: &str = "rebind.example";
const PRIVATE_ANSWER: [u8; 4] = [169, 254, 169, 254];

/// Lookups [`rebinding_dns`] has answered. One test only uses it, so none can race on it.
static REBINDING_LOOKUPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// The first answer is the test server on loopback, public to
/// [`production_filter_except_first`]; every later one is the cloud metadata address.
fn rebinding_dns(_host: String, port: u16) -> BoxFuture<'static, std::io::Result<Vec<SocketAddr>>> {
    let earlier = REBINDING_LOOKUPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let ip = if earlier == 0 {
        Ipv4Addr::LOCALHOST
    } else {
        Ipv4Addr::from(PRIVATE_ANSWER)
    };
    Box::pin(async move { Ok(vec![SocketAddr::from((ip, port))]) })
}

#[tokio::test]
async fn should_refuse_a_name_when_it_rebinds_to_a_private_address() {
    let server = TestServer::start(FIRST, |path| match path {
        "/first" => redirect(302, "/second"),
        _ => ok(b"private"),
    })
    .await;
    let port = Url::parse(&server.url("")).unwrap().port().unwrap();
    let client = vetted_client(
        &hosts(&[REBINDING_HOST]),
        rebinding_dns,
        production_filter_except_first,
        &ProxySource::Direct,
        REQUEST_TIMEOUT,
        None,
        limits().max_redirects,
    )
    .unwrap();
    let get = |path: &str| {
        let url = Url::parse(&format!("http://{REBINDING_HOST}:{port}{path}")).unwrap();
        let request = client.get(url.clone()).send();
        async move { request.await.map_err(|err| send_error(&url, err)) }
    };
    let refused_as_private = |err: &PluginHttpError| {
        matches!(
            err,
            PluginHttpError::BlockedAddress { host, ip }
                if host == REBINDING_HOST && *ip == IpAddr::from(PRIVATE_ANSWER)
        )
    };

    // The redirect stays on the same name, which now resolves privately.
    let err = get("/first")
        .await
        .expect_err("the second answer is private");
    assert!(refused_as_private(&err), "{err}");

    // A later request is checked anew, not let through on the earlier answer.
    let err = get("/second")
        .await
        .expect_err("the name still resolves privately");
    assert!(refused_as_private(&err), "{err}");

    assert_eq!(
        server.requests().len(),
        1,
        "only the hop checked against the public answer reached the server: {:?}",
        server.requests()
    );
    assert_eq!(
        REBINDING_LOOKUPS.load(std::sync::atomic::Ordering::SeqCst),
        3,
        "one lookup per check; the connections never resolved the name themselves"
    );
}
