//! The hosts a plugin may reach, and which hosts the user may be asked to give it.
//!
//! Matching lives here and the address policy in `shared::public_address`, not in the network
//! host functions that enforce them, so that the decision to *give* a host
//! (`GrantPluginHostAccessUseCase`) is made by the very rules the plugin's requests are later
//! held to.

use url::Host;

use super::{PluginManifest, PluginSettings};
pub(crate) use crate::shared::public_address::is_blocked_ip;
use crate::shared::web_url::{HttpUrl, InvalidUrlReason};

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

/// Why an `allowed_hosts` entry typed into the plugin's settings is refused.
///
/// Only an entry that could never match: the settings take a glob (`*.example.com`, even a bare
/// `*`) on purpose, for a plugin of the user's own or a host that moved, so a wide pattern is
/// not refused here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowedHostEntryRejection {
    /// Empty, or only spaces.
    Empty,
    /// A scheme, port, path, credentials or spaces: a host is matched by its name alone, so
    /// such an entry would never match anything.
    NotAHostName,
    /// Not a valid glob pattern.
    InvalidPattern,
}

impl std::fmt::Display for AllowedHostEntryRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Empty => "the entry is empty",
            Self::NotAHostName => {
                "a host is written as its name only, without a scheme, port or path"
            }
            Self::InvalidPattern => "the entry is not a valid pattern",
        })
    }
}

/// Checks one `allowed_hosts` entry of the plugin's settings, and returns it trimmed.
///
/// An IPv6 literal is stored in the glob-escaped form [`GrantableHost`] gives (`[[]::1[]]`), so
/// the colons of an entry that starts that way are not a port.
pub(crate) fn validate_allowed_host_entry(
    entry: &str,
) -> Result<String, AllowedHostEntryRejection> {
    let entry = entry.trim();
    if entry.is_empty() {
        return Err(AllowedHostEntryRejection::Empty);
    }
    let escaped_ipv6 = entry.starts_with("[[]");
    if entry.contains(['/', '\\', '@', '#'])
        || entry.contains(char::is_whitespace)
        || (!escaped_ipv6 && entry.contains(':'))
    {
        return Err(AllowedHostEntryRejection::NotAHostName);
    }
    glob::Pattern::new(entry).map_err(|_| AllowedHostEntryRejection::InvalidPattern)?;
    Ok(entry.to_owned())
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
    ///
    /// Read as an [`HttpUrl`], the rules every web URL of the launcher is held to; surrounding
    /// whitespace, as pasted, is let go.
    pub(crate) fn from_url(raw_url: &str) -> Result<Self, AllowedHostRejection> {
        let url = HttpUrl::parse(raw_url.trim()).map_err(|err| match err.reason {
            InvalidUrlReason::Scheme { .. } => AllowedHostRejection::UnsupportedScheme,
            InvalidUrlReason::NoHost => AllowedHostRejection::MissingHost,
            _ => AllowedHostRejection::InvalidUrl,
        })?;
        if !url.names_public_host() {
            return Err(AllowedHostRejection::NotPublic);
        }

        let host = url.host_str().to_owned();
        let entry = match url.as_url().host() {
            // `url` lets `*` through in a host name; `?` and `[` it does not.
            Some(Host::Domain(name)) if name.contains(['*', '?', '[', ']']) => {
                return Err(AllowedHostRejection::Wildcard);
            }
            // Brackets a glob would read as a character class.
            Some(Host::Ipv6(_)) => glob::Pattern::escape(&host),
            _ => host.clone(),
        };

        Ok(Self { host, entry })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn should_accept_a_host_entry_that_is_a_name_or_a_glob() {
        for (entry, expected) in [
            ("example.com", "example.com"),
            ("  example.com  ", "example.com"),
            ("*.example.com", "*.example.com"),
            ("*", "*"),
            ("*.*", "*.*"),
            ("192.168.1.5", "192.168.1.5"),
            ("[[]::1[]]", "[[]::1[]]"),
        ] {
            assert_eq!(
                validate_allowed_host_entry(entry).as_deref(),
                Ok(expected),
                "{entry}"
            );
        }
    }

    #[test]
    fn should_refuse_a_host_entry_that_could_never_match() {
        for (entry, expected) in [
            ("", AllowedHostEntryRejection::Empty),
            ("   ", AllowedHostEntryRejection::Empty),
            (
                "https://example.com",
                AllowedHostEntryRejection::NotAHostName,
            ),
            (
                "https://example.com:443/",
                AllowedHostEntryRejection::NotAHostName,
            ),
            ("example.com:443", AllowedHostEntryRejection::NotAHostName),
            ("example.com/path", AllowedHostEntryRejection::NotAHostName),
            ("user@example.com", AllowedHostEntryRejection::NotAHostName),
            ("exa mple.com", AllowedHostEntryRejection::NotAHostName),
            ("[abc", AllowedHostEntryRejection::InvalidPattern),
            ("**.example.com", AllowedHostEntryRejection::InvalidPattern),
        ] {
            assert_eq!(validate_allowed_host_entry(entry), Err(expected), "{entry}");
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
