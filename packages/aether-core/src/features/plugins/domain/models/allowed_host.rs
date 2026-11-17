//! The hosts a plugin may reach, and which hosts the user may be asked to give it.
//!
//! Matching lives here and the address policy in `shared::public_address`, not in the network
//! host functions that enforce them, so that the decision to *give* a host
//! (`GrantPluginHostAccessUseCase`) is made by the very rules the plugin's requests are later
//! held to.

use url::Host;

use super::{LocalHost, PluginManifest, PluginSettings};
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

/// Everything a plugin's network may reach: the `allowed_hosts` it matches a host against, and
/// the local hosts the user granted it, whose non-public addresses it may then connect to.
///
/// What the network host functions and the downloads of an install plan hold every request
/// to, read anew for each one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PluginHosts {
    pub allowed: Vec<String>,
    pub local: Vec<LocalHost>,
}

impl PluginHosts {
    pub(crate) fn new(allowed: Vec<String>, local: Vec<LocalHost>) -> Self {
        Self { allowed, local }
    }

    /// The plugin's hosts as the manifest and the user's settings give them: see
    /// [`effective_allowed_hosts`]. Local hosts only come from the settings.
    pub(crate) fn effective(manifest: &PluginManifest, settings: Option<&PluginSettings>) -> Self {
        Self {
            allowed: effective_allowed_hosts(manifest, settings),
            local: settings.map(|s| s.local_hosts.clone()).unwrap_or_default(),
        }
    }

    /// Whether a request may go to `host` on `port` at all: the host is in `allowed_hosts`, or
    /// the user granted it as a local host that is local by how it is written (an address
    /// literal, `localhost`). A granted DNS name must be in `allowed_hosts` as well: the plugin
    /// reports the refusal the grant is offered for, so it must not get a public name it was
    /// never allowed past the allowlist by calling it local.
    pub(crate) fn admits(&self, host: &str, port: u16) -> bool {
        is_host_allowed(host, &self.allowed)
            || self
                .local
                .iter()
                .any(|local| local.is_local_by_name() && local.admits(host, port))
    }

    /// Whether the user let the plugin reach `host` on `port` at a non-public address.
    pub(crate) fn grants_local(&self, host: &str, port: u16) -> bool {
        self.local.iter().any(|local| local.admits(host, port))
    }
}

impl From<Vec<String>> for PluginHosts {
    /// Only `allowed_hosts`, no local host.
    fn from(allowed: Vec<String>) -> Self {
        Self::new(allowed, Vec::new())
    }
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

/// Whether an `allowed_hosts` entry admits far more than one site, so that the user is warned.
///
/// Read the way the matcher reads it ([`is_host_allowed`]): `*` crosses dots, a one-character
/// class such as `[[]` stands for that character (how an IPv6 literal is stored), and an entry
/// that is not a valid glob is compared exactly, so it admits nothing else. An entry with a glob
/// is wide unless the literal text at its end, after the glob, holds at least two whole labels:
/// `*.example.com` stays on one domain, while `*`, `*.com`, `example.*.com`, `*example.com` and
/// `*.com.` (a trailing dot is kept in the host a request names) do not.
///
/// A public suffix under a domain anyone can register (`*.github.io`) is not recognized.
pub fn is_wide_allowed_host(entry: &str) -> bool {
    let entry = entry.trim().to_lowercase();
    if glob::Pattern::new(&entry).is_err() {
        return false;
    }

    // The literal text at the end, after the last glob token.
    let chars: Vec<char> = entry.chars().collect();
    let mut has_glob = false;
    let mut suffix = String::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' | '?' => {
                has_glob = true;
                suffix.clear();
            }
            '[' => {
                // A class as `glob` reads it: after `[` or `[!` the first character belongs to
                // the class even when it is `]`, and the class ends at the next `]`. `Pattern::new`
                // has accepted the entry, so that `]` exists.
                let negated = chars.get(i + 1) == Some(&'!');
                let start = i + 1 + usize::from(negated);
                let end = (start + 1..chars.len())
                    .find(|&j| chars[j] == ']')
                    .unwrap_or(chars.len() - 1);
                if !negated && end == start + 1 {
                    suffix.push(chars[start]);
                } else {
                    has_glob = true;
                    suffix.clear();
                }
                i = end;
            }
            c => suffix.push(c),
        }
        i += 1;
    }

    if !has_glob {
        return false;
    }

    let suffix = suffix.strip_suffix('.').unwrap_or(&suffix);
    // The first label touches the glob, so the glob may extend it: it is not a whole label.
    let whole_labels = suffix
        .split('.')
        .skip(1)
        .filter(|label| !label.is_empty())
        .count();

    whole_labels < 2
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
    /// Given as a local host, an address literal that is public (`allowed_hosts` gives it), or
    /// one that is no single host (unspecified, broadcast, multicast).
    NotLocal,
    /// Given as a local host, a name that could not be resolved, or resolved to no address.
    Unresolved,
}

impl std::fmt::Display for AllowedHostRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidUrl => "not a valid URL",
            Self::UnsupportedScheme => "only http and https URLs are supported",
            Self::MissingHost => "the URL has no host",
            Self::NotPublic => "the host is local or not a public address",
            Self::Wildcard => "the host name contains a wildcard",
            Self::NotLocal => "the address is not one local host",
            Self::Unresolved => "the host name does not resolve",
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

/// Checks one `allowed_hosts` entry of the plugin's settings, and returns it trimmed and
/// lowercased.
///
/// An IPv6 literal is stored in the glob-escaped form [`GrantableHost`] gives (`[[]::1[]]`), so
/// the colons of an entry that starts that way are not a port.
pub(crate) fn validate_allowed_host_entry(
    entry: &str,
) -> Result<String, AllowedHostEntryRejection> {
    // Lowercased: the host a request names is, and a glob matches case-sensitively.
    let entry = entry.trim().to_lowercase();
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
    glob::Pattern::new(&entry).map_err(|_| AllowedHostEntryRejection::InvalidPattern)?;
    Ok(entry)
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

    #[test]
    fn should_flag_a_pattern_that_admits_far_more_than_one_site_as_wide() {
        for entry in [
            "*",
            "*.*",
            "*.com",
            "*.com.",
            "example.*.com",
            "*example.com",
            "ex*mple.com",
            "[[]*",
            "1.2.3.*",
            "*.COM",
            "[!a].com",
        ] {
            assert!(is_wide_allowed_host(entry), "{entry} should be wide");
        }
    }

    #[test]
    fn should_not_flag_a_host_or_a_pattern_that_stays_on_one_domain_as_wide() {
        for entry in [
            "example.com",
            "*.example.com",
            "?.example.com",
            "*.api.example.com",
            "[[]2001:db8::1[]]",
            "[[]::1[]]",
            "[abc",
            "",
        ] {
            assert!(!is_wide_allowed_host(entry), "{entry} should not be wide");
        }
    }

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
            ("Example.COM", "example.com"),
            ("*.GitHub.com", "*.github.com"),
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
