//! The local addresses the user let one plugin reach: a server in their own network, a NAS, a
//! service on this machine.
//!
//! A plugin's requests are refused every address that is not public
//! (`shared::public_address`). A grant lifts that for one host, and one port when it names one,
//! for one plugin, and admits that host past its `allowed_hosts` too. Only the user makes one
//! (`GrantPluginLocalHostAccessUseCase`, behind a confirmation), never a manifest, and never by
//! a pattern: a grant names one host as the plugin's requests name it.

use std::{fmt, net::IpAddr};

use url::{Host, Url};

use super::AllowedHostRejection;
use crate::shared::{
    public_address::{is_blocked_ip, is_never_grantable},
    web_url::{HttpUrl, InvalidUrlReason},
};

/// One local host a plugin may reach: `host`, or `host:port`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocalHost {
    /// As the network host functions see it (`Url::host_str`): lowercased, an IPv6 literal in
    /// brackets.
    host: String,
    /// `None`: any port of the host.
    port: Option<u16>,
}

/// Why a stored local host entry cannot be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("not a host or host:port, written as a request names it")]
pub struct InvalidLocalHost;

impl LocalHost {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Whether a request to `host` (as `Url::host_str` gives it) on `port` is this grant's.
    pub(crate) fn admits(&self, host: &str, port: u16) -> bool {
        self.host == host && self.port.is_none_or(|own| own == port)
    }

    /// The grant for the host of `raw_url` and its port (the scheme's when the URL names none).
    ///
    /// Refused: not `http(s)`, no host, a wildcard in the name, and an address literal that is
    /// public (it needs no grant: `allowed_hosts` gives it) or that no grant may name
    /// ([`is_never_grantable`]). A name is not resolved here: what it resolves to is judged when
    /// the plugin's requests are made.
    pub(crate) fn from_url(raw_url: &str) -> Result<Self, AllowedHostRejection> {
        let url = HttpUrl::parse(raw_url.trim()).map_err(|err| match err.reason {
            InvalidUrlReason::Scheme { .. } => AllowedHostRejection::UnsupportedScheme,
            InvalidUrlReason::NoHost => AllowedHostRejection::MissingHost,
            _ => AllowedHostRejection::InvalidUrl,
        })?;
        let url = url.as_url();
        Self::check_host(url)
            .map_err(|reason| reason.unwrap_or(AllowedHostRejection::InvalidUrl))?;

        Ok(Self {
            host: url.host_str().unwrap_or_default().to_owned(),
            port: url.port_or_known_default(),
        })
    }

    /// `Err(None)` when the URL has no host at all.
    fn check_host(url: &Url) -> Result<(), Option<AllowedHostRejection>> {
        let literal = match url.host().ok_or(None)? {
            // `url` lets `*` through in a host name; `?` and `[` it does not.
            Host::Domain(name) if name.contains(['*', '?', '[', ']']) => {
                return Err(Some(AllowedHostRejection::Wildcard));
            }
            Host::Domain(_) => return Ok(()),
            Host::Ipv4(ip) => IpAddr::V4(ip),
            Host::Ipv6(ip) => IpAddr::V6(ip),
        };
        if !is_blocked_ip(literal) || is_never_grantable(literal) {
            return Err(Some(AllowedHostRejection::NotLocal));
        }
        Ok(())
    }

    /// An entry as [`Self::fmt`] writes it, which is how the plugin's settings store it. Only
    /// the exact form is taken back: an entry written any other way is not this launcher's.
    pub(crate) fn parse(entry: &str) -> Result<Self, InvalidLocalHost> {
        // The port is after the last `:` outside an IPv6 literal's brackets.
        let (host, port) = match entry.rfind(':') {
            Some(colon) if !entry[colon..].contains(']') => {
                let digits = &entry[colon + 1..];
                if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(InvalidLocalHost);
                }
                let port = digits.parse::<u16>().map_err(|_| InvalidLocalHost)?;
                (&entry[..colon], Some(port))
            }
            _ => (entry, None),
        };
        if host.is_empty() || port == Some(0) {
            return Err(InvalidLocalHost);
        }

        let url = Url::parse(&format!("http://{host}/")).map_err(|_| InvalidLocalHost)?;
        if url.host_str() != Some(host) || Self::check_host(&url).is_err() {
            return Err(InvalidLocalHost);
        }

        Ok(Self {
            host: host.to_owned(),
            port,
        })
    }
}

impl fmt::Display for LocalHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.port {
            Some(port) => write!(f, "{}:{port}", self.host),
            None => f.write_str(&self.host),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_name_the_host_and_its_port_when_granted_from_a_url() {
        for (url, expected) in [
            ("http://192.168.1.10:8080/pack.toml", "192.168.1.10:8080"),
            ("http://192.168.1.10/pack.toml", "192.168.1.10:80"),
            ("https://NAS.lan/packs/", "nas.lan:443"),
            ("http://localhost:25565/", "localhost:25565"),
            ("  http://[::1]:8080/  ", "[::1]:8080"),
            ("http://[fd12::1]/", "[fd12::1]:80"),
        ] {
            let local = LocalHost::from_url(url).unwrap();

            assert_eq!(local.to_string(), expected, "{url}");
            assert_eq!(LocalHost::parse(expected), Ok(local), "{url}");
        }
    }

    #[test]
    fn should_refuse_a_grant_when_it_names_no_single_local_host() {
        for (url, expected) in [
            ("not a url", AllowedHostRejection::InvalidUrl),
            (
                "ftp://192.168.1.10/",
                AllowedHostRejection::UnsupportedScheme,
            ),
            ("http:///pack.toml", AllowedHostRejection::MissingHost),
            ("http://*.lan/", AllowedHostRejection::Wildcard),
            ("http://8.8.8.8/", AllowedHostRejection::NotLocal),
            (
                "http://[2606:4700:4700::1111]/",
                AllowedHostRejection::NotLocal,
            ),
            ("http://0.0.0.0:8080/", AllowedHostRejection::NotLocal),
            ("http://[::]/", AllowedHostRejection::NotLocal),
            ("http://255.255.255.255/", AllowedHostRejection::NotLocal),
            ("http://224.0.0.1/", AllowedHostRejection::NotLocal),
            ("http://[ff02::1]/", AllowedHostRejection::NotLocal),
        ] {
            assert_eq!(LocalHost::from_url(url), Err(expected), "{url}");
        }
    }

    #[test]
    fn should_read_back_only_the_form_it_writes() {
        for entry in [
            "localhost",
            "nas.lan:8080",
            "10.0.0.2",
            "[::1]",
            "[fe80::1]:443",
        ] {
            assert_eq!(
                LocalHost::parse(entry)
                    .map(|local| local.to_string())
                    .as_deref(),
                Ok(entry),
                "{entry}"
            );
        }
        for entry in [
            "",
            ":80",
            "NAS.lan",
            "nas.lan:",
            "nas.lan:0",
            "nas.lan:65536",
            "nas.lan:+80",
            "nas.lan:80:80",
            "::1",
            "[::1]:",
            "*.lan",
            "nas.lan/x",
            "user@nas.lan",
            "8.8.8.8",
            "0.0.0.0",
            "0x7f.1",
        ] {
            assert_eq!(LocalHost::parse(entry), Err(InvalidLocalHost), "{entry}");
        }
    }

    #[test]
    fn should_admit_only_its_own_host_and_port() {
        let with_port = LocalHost::parse("192.168.1.10:8080").unwrap();
        assert!(with_port.admits("192.168.1.10", 8080));
        assert!(!with_port.admits("192.168.1.10", 80));
        assert!(!with_port.admits("192.168.1.11", 8080));

        let any_port = LocalHost::parse("localhost").unwrap();
        assert!(any_port.admits("localhost", 8080));
        assert!(any_port.admits("localhost", 443));
        assert!(!any_port.admits("api.localhost", 443));
        assert!(!any_port.admits("127.0.0.1", 443));
    }
}
