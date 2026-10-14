//! The address and redirect policy behind `http_get` / `http_post_json`: vetting a host, pinning
//! the addresses that passed, and the hop loop that vets every redirect.

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex, Once},
    time::{Duration, Instant},
};

use hyper_util::client::proxy::matcher::{Intercept, Matcher as ProxyMatcher};
use reqwest::{
    StatusCode,
    dns::{Addrs, Name, Resolve, Resolving},
    header::{HeaderValue, LOCATION},
};
use reqwest_middleware::{Middleware, Next};
use url::Url;

use super::{AddressFilter, HostLookup, PluginHttpError, check_target};

/// The redirect statuses a GET follows. Any other 3xx (`300`, `304`, …) is not a hop: it is
/// handed back to the caller, which treats it as an error.
const FOLLOWED_REDIRECTS: &[StatusCode] = &[
    StatusCode::MOVED_PERMANENTLY,
    StatusCode::FOUND,
    StatusCode::SEE_OTHER,
    StatusCode::TEMPORARY_REDIRECT,
    StatusCode::PERMANENT_REDIRECT,
];

/// Resolve `host:port` and make sure **every** returned address is public.
///
/// All addresses must pass: a name that resolves to one public and one private address is
/// rejected outright, since we cannot control which one a later connect would pick.
pub(super) async fn resolve_public_addrs(
    host: &str,
    port: u16,
    lookup: HostLookup,
    is_blocked: AddressFilter,
) -> Result<Vec<SocketAddr>, PluginHttpError> {
    let addrs =
        lookup(host.to_owned(), port)
            .await
            .map_err(|e| PluginHttpError::ResolveFailed {
                host: host.to_owned(),
                reason: e.to_string(),
            })?;

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

// ── Pinned resolution and vetted hops ──

/// The only DNS a vetted client has: the names [`VettedHops`] has just checked, each pinned to
/// exactly the addresses that passed the check.
///
/// A name that was not vetted fails to resolve (fail closed), so the connection never goes to a
/// second, possibly private, DNS answer. IP literals never reach a resolver; the middleware vets
/// them before the request is sent.
#[derive(Default)]
pub(super) struct PinnedResolver {
    pins: Mutex<HashMap<String, Vec<SocketAddr>>>,
}

impl PinnedResolver {
    pub(super) fn pin(&self, host: String, addrs: Vec<SocketAddr>) {
        if let Ok(mut pins) = self.pins.lock() {
            pins.insert(host, addrs);
        }
    }
}

impl Resolve for PinnedResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let addrs = self
            .pins
            .lock()
            .ok()
            .and_then(|pins| pins.get(name.as_str()).cloned());

        Box::pin(async move {
            match addrs {
                Some(addrs) => Ok(Box::new(addrs.into_iter()) as Addrs),
                None => Err(format!("host `{}` has not been vetted", name.as_str()).into()),
            }
        })
    }
}

/// Where a redirect leads: its `Location`, resolved against the URL that answered.
///
/// Refuses a missing or unparseable `Location` and a downgrade from `https` to `http`. The
/// target still has to pass [`VettedHops::vet`], like a first request.
pub(super) fn redirect_target(
    from: &Url,
    status: StatusCode,
    location: Option<&HeaderValue>,
) -> Result<Url, PluginHttpError> {
    let invalid = |reason: String| PluginHttpError::InvalidRedirect {
        status: status.as_u16(),
        url: from.to_string(),
        reason,
    };

    let location = location
        .ok_or_else(|| invalid("the header is missing".to_owned()))?
        .to_str()
        .map_err(|e| invalid(e.to_string()))?;
    let target = from.join(location).map_err(|e| invalid(e.to_string()))?;

    if from.scheme() == "https" && target.scheme() == "http" {
        return Err(PluginHttpError::SchemeDowngrade {
            from: from.to_string(),
            to: target.to_string(),
        });
    }

    Ok(target)
}

/// Logs, once per process, that plugin requests go through a proxy.
static PROXY_NOTICE: Once = Once::new();

/// The hop loop of every host-side request a plugin causes.
///
/// Each hop, the first included, is vetted before it is sent. A GET follows
/// [`FOLLOWED_REDIRECTS`] up to `max_redirects` times; any other method gets its 3xx back
/// untouched. The time budget covers the whole chain, not each hop.
pub(super) struct VettedHops {
    pub(super) allowed_hosts: Vec<String>,
    pub(super) lookup: HostLookup,
    pub(super) is_blocked: AddressFilter,
    /// Whether the client sends a hop through a proxy: the settings the client was built with.
    pub(super) proxies: ProxyMatcher,
    pub(super) resolver: Arc<PinnedResolver>,
    pub(super) total_timeout: Duration,
    /// Most redirects a GET follows. Being redirected once more is an error.
    pub(super) max_redirects: usize,
}

impl VettedHops {
    /// Check `url` exactly like a first request and pin the addresses that passed. Nothing is
    /// sent to the host before this succeeds.
    async fn vet(&self, url: &Url) -> Result<(), PluginHttpError> {
        let (url, host) = check_target(url.as_str(), &self.allowed_hosts)?;

        if let Some(proxy) = self.proxy_for(&url) {
            return self.vet_proxied(&url, &host, &proxy).await;
        }

        let port = url
            .port_or_known_default()
            .unwrap_or(if url.scheme() == "https" { 443 } else { 80 });
        let addrs = resolve_public_addrs(&host, port, self.lookup, self.is_blocked).await?;

        self.resolver.pin(host, addrs);
        Ok(())
    }

    /// The proxy the client will send `url` through, if any. A URL the proxy rules cannot even
    /// read is taken as direct, which is vetted the strict way.
    fn proxy_for(&self, url: &Url) -> Option<Intercept> {
        let uri: http::Uri = url.as_str().parse().ok()?;
        self.proxies.intercept(&uri)
    }

    /// A hop through a proxy, which resolves `host` itself: its addresses are neither checked
    /// nor pinned, but an IP-literal host is still filtered. The proxy is not filtered (the user
    /// configured it, not the plugin), only pinned so the client's resolver accepts it.
    async fn vet_proxied(
        &self,
        url: &Url,
        host: &str,
        proxy: &Intercept,
    ) -> Result<(), PluginHttpError> {
        let literal = match url.host() {
            Some(url::Host::Ipv4(ip)) => Some(IpAddr::V4(ip)),
            Some(url::Host::Ipv6(ip)) => Some(IpAddr::V6(ip)),
            _ => None,
        };
        if let Some(ip) = literal
            && (self.is_blocked)(ip)
        {
            return Err(PluginHttpError::BlockedAddress {
                host: host.to_owned(),
                ip,
            });
        }

        let proxy_uri = proxy.uri();
        let Some(proxy_host) = proxy_uri.host() else {
            return Ok(());
        };
        let default_port = if proxy_uri.scheme_str() == Some("https") {
            443
        } else {
            80
        };
        let proxy_port = proxy_uri.port_u16().unwrap_or(default_port);

        PROXY_NOTICE.call_once(|| {
            log::info!(
                target: "plugin",
                "Plugin requests go through the configured proxy {proxy_host}:{proxy_port}; \
                 the proxy resolves host names, so it, not the launcher, checks their addresses"
            );
        });

        let bare = proxy_host.trim_start_matches('[').trim_end_matches(']');
        if bare.parse::<IpAddr>().is_err() {
            let addrs = (self.lookup)(proxy_host.to_owned(), proxy_port)
                .await
                .map_err(|e| PluginHttpError::ResolveFailed {
                    host: proxy_host.to_owned(),
                    reason: e.to_string(),
                })?;
            self.resolver.pin(proxy_host.to_owned(), addrs);
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Middleware for VettedHops {
    async fn handle(
        &self,
        mut request: reqwest::Request,
        extensions: &mut http::Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        let deadline = Instant::now() + self.total_timeout;
        let follows = request.method() == reqwest::Method::GET;
        let mut redirects = 0;

        loop {
            self.vet(request.url())
                .await
                .map_err(reqwest_middleware::Error::middleware)?;

            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(reqwest_middleware::Error::middleware(
                    PluginHttpError::RequestFailed {
                        url: request.url().to_string(),
                        reason: "timed out while following redirects".to_owned(),
                    },
                ));
            }
            *request.timeout_mut() = Some(remaining);

            let url = request.url().clone();
            let headers = request.headers().clone();
            let response = next.clone().run(request, extensions).await?;

            let status = response.status();
            if !follows || !FOLLOWED_REDIRECTS.contains(&status) {
                return Ok(response);
            }
            if redirects == self.max_redirects {
                return Err(reqwest_middleware::Error::middleware(
                    PluginHttpError::TooManyRedirects {
                        limit: self.max_redirects,
                    },
                ));
            }

            let target = redirect_target(&url, status, response.headers().get(LOCATION))
                .map_err(reqwest_middleware::Error::middleware)?;
            // The redirect's own body is never read: dropping the response discards it.
            drop(response);

            // The plugin's headers (an API key, say) are meant for the origin it asked for.
            // Once the chain leaves it they are dropped, and stay dropped for later hops.
            let same_origin = target.origin() == url.origin();
            request = reqwest::Request::new(reqwest::Method::GET, target);
            if same_origin {
                *request.headers_mut() = headers;
            }
            redirects += 1;
        }
    }
}

/// Unwrap a policy refusal raised by [`VettedHops`], so callers see the same
/// [`PluginHttpError`] for any hop as for the first request. Anything else is handed back.
pub(in super::super) fn policy_error(err: anyhow::Error) -> Result<PluginHttpError, anyhow::Error> {
    find_policy_error(&err).cloned().ok_or(err)
}

/// The retry middleware re-wraps every error transparently, so the refusal may sit under it
/// without showing up in the `source()` chain.
fn find_policy_error(err: &anyhow::Error) -> Option<&PluginHttpError> {
    if let Some(policy) = err.downcast_ref::<PluginHttpError>() {
        return Some(policy);
    }

    match err.downcast_ref::<reqwest_retry::RetryError>()? {
        reqwest_retry::RetryError::Error(reqwest_middleware::Error::Middleware(inner))
        | reqwest_retry::RetryError::WithRetries {
            err: reqwest_middleware::Error::Middleware(inner),
            ..
        } => find_policy_error(inner),
        _ => None,
    }
}
