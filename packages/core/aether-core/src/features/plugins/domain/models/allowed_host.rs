//! The hosts a plugin may reach, and which hosts the user may be asked to give it.
//!
//! Matching and the address policy live here, not in the network host functions that enforce
//! them, so that the decision to *give* a host (`GrantPluginHostAccessUseCase`) is made by the
//! very rules the plugin's requests are later held to.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use url::{Host, Url};

use super::{PluginManifest, PluginSettings};

/// The hosts a plugin may reach: its manifest's `allowed_hosts`, then those the user added in
/// its settings.
///
/// One list for everything a plugin's network goes through — the Extism manifest, the network
/// host functions, and the downloads of the URLs it puts into an install plan — so none of them
/// can let through a host another refuses.
pub(crate) fn effective_allowed_hosts(
    manifest: &PluginManifest,
    settings: Option<&PluginSettings>,
) -> Vec<String> {
    let mut allowed_hosts = manifest.runtime.allowed_hosts.clone();

    if let Some(settings) = settings {
        allowed_hosts.extend_from_slice(&settings.allowed_hosts);
    }

    allowed_hosts
}

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

/// An allowlist, not a denylist: only global unicast (`2000::/3`) may pass, less the ranges in it
/// that are not public, and the forms that carry an IPv4 address the connection really goes to
/// are judged by that address.
fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    // An IPv4 address wearing an IPv6 costume must be judged as IPv4.
    if let Some(v4) = embedded_ipv4(ip) {
        return is_blocked_ipv4(v4);
    }

    let s = ip.segments();

    // Everything outside 2000::/3 is not public: `::`, `::1`, the deprecated IPv4-compatible
    // `::a.b.c.d` and SIIT `::ffff:0:a.b.c.d` forms, 100::/64 (discard), the local-use NAT64
    // prefix 64:ff9b:1::/48 (where its IPv4 sits depends on a prefix length only the network
    // knows), fc00::/7 (unique local), fe80::/10 (link-local), fec0::/10 (site-local),
    // ff00::/8 (multicast) and whatever IANA has yet to assign.
    if (s[0] & 0xe000) != 0x2000 {
        return true;
    }

    // 2001::/32 — Teredo
    (s[0] == 0x2001 && s[1] == 0)
        // 2001:2::/48 — benchmarking
        || (s[0] == 0x2001 && s[1] == 0x0002 && s[2] == 0)
        // 2001:10::/28 — ORCHID, 2001:20::/28 — ORCHIDv2
        || (s[0] == 0x2001 && (s[1] & 0xfff0) == 0x0010)
        || (s[0] == 0x2001 && (s[1] & 0xfff0) == 0x0020)
        // 2001:db8::/32 — documentation
        || (s[0] == 0x2001 && s[1] == 0x0db8)
        // 3fff::/20 — documentation
        || (s[0] & 0xfff0) == 0x3ff0
        // 3ffe::/16 — the former 6bone, returned to IANA
        || s[0] == 0x3ffe
}

/// The IPv4 address an IPv6 address stands for, where the connection to it reaches that IPv4
/// address: `::ffff:a.b.c.d` (IPv4-mapped, which a dual-stack socket connects to over IPv4),
/// `64:ff9b::a.b.c.d` (the well-known NAT64 prefix) and `2002:AABB:CCDD::/48` (6to4).
fn embedded_ipv4(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let s = ip.segments();
    let v4 = |hi: u16, lo: u16| Ipv4Addr::from((u32::from(hi) << 16) | u32::from(lo));

    if let Some(v4) = ip.to_ipv4_mapped() {
        return Some(v4);
    }
    // 64:ff9b::/96 — NAT64 well-known prefix (RFC 6052)
    if s[..6] == [0x0064, 0xff9b, 0, 0, 0, 0] {
        return Some(v4(s[6], s[7]));
    }
    // 2002::/16 — 6to4 (RFC 3056)
    if s[0] == 0x2002 {
        return Some(v4(s[1], s[2]));
    }
    None
}

/// Why the host of a URL cannot be given to a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowedHostRejection {
    /// Not a URL at all.
    InvalidUrl,
    /// Not `http` or `https`, the only schemes a plugin's requests may use.
    UnsupportedScheme,
    /// The URL names no host.
    MissingHost,
    /// `localhost`, or an address that is not public: the plugin's requests would be refused
    /// there whatever its `allowed_hosts` say.
    NotPublic,
    /// The host name holds a glob wildcard: as an `allowed_hosts` entry it would admit more
    /// than the host the user saw.
    Wildcard,
}

impl std::fmt::Display for AllowedHostRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidUrl => "not a valid URL",
            Self::UnsupportedScheme => "only http and https URLs are supported",
            Self::MissingHost => "the URL has no host",
            Self::NotPublic => "the host is local or not a public address",
            Self::Wildcard => "the host name contains a wildcard",
        })
    }
}

/// A host the user may give a plugin for `raw_url`: the URL's host as the plugin's requests will
/// name it, and the `allowed_hosts` entry that admits that host and nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GrantableHost {
    /// As the network host functions see it (`Url::host_str`): what [`is_host_allowed`] is
    /// asked about.
    pub host: String,
    /// The `allowed_hosts` entry to add. The host itself, but for an IPv6 literal, whose
    /// brackets a glob would read as a character class.
    pub entry: String,
}

impl GrantableHost {
    /// The host of `raw_url`, refused where the plugin's requests would be refused anyway: not
    /// `http(s)`, no host, `localhost` or a non-public address literal (the address policy of
    /// `http_get` and `download_to_cache`). A host *name* that resolves to a private address is
    /// still refused, but only by those requests: nothing is resolved here.
    pub(crate) fn from_url(raw_url: &str) -> Result<Self, AllowedHostRejection> {
        let url = Url::parse(raw_url.trim()).map_err(|_| AllowedHostRejection::InvalidUrl)?;

        if !matches!(url.scheme(), "http" | "https") {
            return Err(AllowedHostRejection::UnsupportedScheme);
        }

        let host = url.host_str().unwrap_or_default().to_owned();

        let entry = match url.host() {
            None => return Err(AllowedHostRejection::MissingHost),
            Some(Host::Domain(name)) => {
                if name.is_empty() {
                    return Err(AllowedHostRejection::MissingHost);
                }
                if is_localhost(name) {
                    return Err(AllowedHostRejection::NotPublic);
                }
                // `url` lets `*` through in a host name; `?` and `[` it does not.
                if name.contains(['*', '?', '[', ']']) {
                    return Err(AllowedHostRejection::Wildcard);
                }
                host.clone()
            }
            Some(Host::Ipv4(ip)) => {
                if is_blocked_ip(IpAddr::V4(ip)) {
                    return Err(AllowedHostRejection::NotPublic);
                }
                host.clone()
            }
            Some(Host::Ipv6(ip)) => {
                if is_blocked_ip(IpAddr::V6(ip)) {
                    return Err(AllowedHostRejection::NotPublic);
                }
                glob::Pattern::escape(&host)
            }
        };

        Ok(Self { host, entry })
    }
}

/// `localhost` and its subdomains always mean the local machine (RFC 6761).
fn is_localhost(name: &str) -> bool {
    let name = name.trim_end_matches('.');
    name == "localhost" || name.ends_with(".localhost")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grantable(url: &str) -> Result<GrantableHost, AllowedHostRejection> {
        GrantableHost::from_url(url)
    }

    #[test]
    fn should_take_the_host_when_the_url_is_http() {
        let host = grantable("https://Example.com:8443/packs/pack.toml?x=1").unwrap();

        assert_eq!(host.host, "example.com");
        assert_eq!(host.entry, "example.com");
        assert!(is_host_allowed(&host.host, &[host.entry]));
    }

    #[test]
    fn should_refuse_a_host_when_the_plugin_could_not_request_it() {
        for (url, expected) in [
            ("not a url", AllowedHostRejection::InvalidUrl),
            (
                "D:\\packs\\pack.toml",
                AllowedHostRejection::UnsupportedScheme,
            ),
            (
                "file:///packs/pack.toml",
                AllowedHostRejection::UnsupportedScheme,
            ),
            (
                "ftp://example.com/pack.toml",
                AllowedHostRejection::UnsupportedScheme,
            ),
            (
                "http://localhost:8080/pack.toml",
                AllowedHostRejection::NotPublic,
            ),
            (
                "http://api.localhost/pack.toml",
                AllowedHostRejection::NotPublic,
            ),
            (
                "http://127.0.0.1/pack.toml",
                AllowedHostRejection::NotPublic,
            ),
            (
                "http://2130706433/pack.toml",
                AllowedHostRejection::NotPublic,
            ),
            (
                "http://192.168.1.10/pack.toml",
                AllowedHostRejection::NotPublic,
            ),
            (
                "http://169.254.169.254/latest",
                AllowedHostRejection::NotPublic,
            ),
            ("http://[::1]/pack.toml", AllowedHostRejection::NotPublic),
            (
                "http://[::ffff:10.0.0.1]/pack.toml",
                AllowedHostRejection::NotPublic,
            ),
            (
                "http://*.example.com/pack.toml",
                AllowedHostRejection::Wildcard,
            ),
        ] {
            assert_eq!(grantable(url), Err(expected), "{url}");
        }
    }

    fn blocked(ip: &str) -> bool {
        is_blocked_ip(ip.parse().unwrap())
    }

    #[test]
    fn should_judge_by_the_embedded_ipv4_when_the_address_is_nat64() {
        assert!(blocked("64:ff9b::7f00:1"), "127.0.0.1");
        assert!(blocked("64:ff9b::10.0.0.1"));
        assert!(blocked("64:ff9b::169.254.169.254"));
        assert!(!blocked("64:ff9b::8.8.8.8"));
    }

    #[test]
    fn should_block_the_whole_prefix_when_the_address_is_local_use_nat64() {
        assert!(blocked("64:ff9b:1::10.0.0.1"));
        assert!(blocked("64:ff9b:1::8.8.8.8"));
        assert!(blocked("64:ff9b:1:a00:1::"));
    }

    #[test]
    fn should_judge_by_the_embedded_ipv4_when_the_address_is_6to4() {
        assert!(blocked("2002:7f00:1::1"), "127.0.0.1");
        assert!(blocked("2002:c0a8:101::1"), "192.168.1.1");
        assert!(blocked("2002:a9fe:a9fe::"), "169.254.169.254");
        assert!(!blocked("2002:808:808::1"), "8.8.8.8");
    }

    #[test]
    fn should_block_when_the_address_is_site_local() {
        assert!(blocked("fec0::1"));
        assert!(blocked("feff:ffff::1"));
    }

    #[test]
    fn should_block_when_the_address_is_discard_only() {
        assert!(blocked("100::"));
        assert!(blocked("100::ffff:ffff:ffff:ffff"));
    }

    #[test]
    fn should_judge_by_the_embedded_ipv4_when_the_address_is_ipv4_mapped() {
        assert!(blocked("::ffff:127.0.0.1"));
        assert!(blocked("::ffff:100.64.0.1"));
        assert!(!blocked("::ffff:8.8.8.8"));
    }

    #[test]
    fn should_block_when_the_address_is_ipv4_compatible_or_translated() {
        assert!(blocked("::"));
        assert!(blocked("::1"));
        assert!(blocked("::10.0.0.1"));
        assert!(blocked("::8.8.8.8"), "deprecated, never routed as IPv4");
        assert!(blocked("::ffff:0:8.8.8.8"), "SIIT");
    }

    #[test]
    fn should_block_when_the_address_is_outside_global_unicast() {
        for ip in [
            "1::1", "4000::1", "fc00::1", "fd12::1", "fe80::1", "ff02::1",
        ] {
            assert!(blocked(ip), "{ip}");
        }
    }

    #[test]
    fn should_block_when_the_address_is_special_purpose_global_unicast() {
        for ip in [
            "2001::1",
            "2001:0:4136:e378:8000:63bf:3fff:fdd2",
            "2001:2::1",
            "2001:10::1",
            "2001:20::1",
            "2001:db8::1",
            "3fff::1",
            "3ffe::1",
        ] {
            assert!(blocked(ip), "{ip}");
        }
    }

    #[test]
    fn should_allow_when_the_address_is_public_global_unicast() {
        for ip in [
            "2606:4700:4700::1111",
            "2001:4860:4860::8888",
            "2a00:1450:4001::1",
            "2001:200::1",
        ] {
            assert!(!blocked(ip), "{ip}");
        }
    }

    #[test]
    fn should_give_the_address_when_the_host_is_a_public_address_literal() {
        let v4 = grantable("http://8.8.8.8/pack.toml").unwrap();
        assert_eq!(v4.entry, "8.8.8.8");
        assert!(is_host_allowed(&v4.host, &[v4.entry]));

        let v6 = grantable("http://[2606:4700:4700::1111]/pack.toml").unwrap();
        assert_eq!(v6.host, "[2606:4700:4700::1111]");
        assert!(is_host_allowed(&v6.host, std::slice::from_ref(&v6.entry)));
        assert!(
            !is_host_allowed("1", &[v6.entry]),
            "brackets read literally"
        );
    }
}
