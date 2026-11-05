//! The hosts a plugin may reach, and which hosts the user may be asked to give it.
//!
//! Matching lives here and the address policy in `shared::public_address`, not in the network
//! host functions that enforce them, so that the decision to *give* a host
//! (`GrantPluginHostAccessUseCase`) is made by the very rules the plugin's requests are later
//! held to.

use std::net::IpAddr;

use url::{Host, Url};

use super::{PluginManifest, PluginSettings};
pub(crate) use crate::shared::public_address::is_blocked_ip;
use crate::shared::public_address::is_localhost;

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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn grantable(url: &str) -> Result<GrantableHost, AllowedHostRejection> {
        GrantableHost::from_url(url)
    }

    #[test]
    fn should_take_the_host_when_the_url_has_a_scheme_port_and_query() {
        for url in [
            "https://Example.com:8443/packs/pack.toml?x=1",
            "http://Example.com:8443/packs/pack.toml?x=1",
        ] {
            let host = grantable(url).unwrap();

            assert_eq!(host.host, "example.com", "{url}");
            assert_eq!(host.entry, "example.com", "{url}");
            assert!(is_host_allowed("example.com", &["example.com".to_owned()]));
        }
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

    #[rstest]
    #[case::nat64_embedded_ipv4(
        &["64:ff9b::7f00:1", "64:ff9b::10.0.0.1", "64:ff9b::169.254.169.254"],
        &["64:ff9b::8.8.8.8"]
    )]
    #[case::local_use_nat64_whole_prefix(
        &["64:ff9b:1::10.0.0.1", "64:ff9b:1::8.8.8.8", "64:ff9b:1:a00:1::"],
        &[]
    )]
    #[case::six_to_four_embedded_ipv4(
        // 127.0.0.1, 192.168.1.1, 169.254.169.254, then 8.8.8.8 allowed.
        &["2002:7f00:1::1", "2002:c0a8:101::1", "2002:a9fe:a9fe::"],
        &["2002:808:808::1"]
    )]
    #[case::site_local(&["fec0::1", "feff:ffff::1"], &[])]
    #[case::discard_only(&["100::", "100::ffff:ffff:ffff:ffff"], &[])]
    #[case::ipv4_mapped(
        &["::ffff:127.0.0.1", "::ffff:100.64.0.1"],
        &["::ffff:8.8.8.8"]
    )]
    #[case::ipv4_compatible_or_translated(
        // The last two are deprecated (never routed as IPv4) and SIIT.
        &["::", "::1", "::10.0.0.1", "::8.8.8.8", "::ffff:0:8.8.8.8"],
        &[]
    )]
    #[case::outside_global_unicast(
        &["1::1", "4000::1", "fc00::1", "fd12::1", "fe80::1", "ff02::1"],
        &[]
    )]
    #[case::special_purpose_global_unicast(
        &[
            "2001::1",
            "2001:0:4136:e378:8000:63bf:3fff:fdd2",
            "2001:2::1",
            "2001:10::1",
            "2001:20::1",
            "2001:db8::1",
            "3fff::1",
            "3ffe::1",
        ],
        &[]
    )]
    #[case::public_global_unicast(
        &[],
        &[
            "2606:4700:4700::1111",
            "2001:4860:4860::8888",
            "2a00:1450:4001::1",
            "2001:200::1",
        ]
    )]
    fn should_judge_the_ipv6_address(#[case] blocked_ips: &[&str], #[case] allowed_ips: &[&str]) {
        for ip in blocked_ips {
            assert!(blocked(ip), "{ip}");
        }
        for ip in allowed_ips {
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
