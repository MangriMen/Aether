//! URLs of the web, checked once where they come in: [`HttpsUrl`] for what the launcher fetches
//! itself (downloads, icons), [`HttpUrl`] for pages it only links to.
//!
//! Both refuse what a lenient parser would quietly repair (surrounding or inner whitespace,
//! backslashes, a missing `//`), so the URL checked is the URL fetched or shown, and any
//! credentials in the authority, which would end up in logs.

use std::{fmt, net::IpAddr, str::FromStr};

use url::{Host, Url};

use super::public_address::{is_blocked_ip, is_localhost};

/// The longest URL taken, in bytes. Far above any download or icon URL of a real source,
/// signed CDN URLs included.
pub const MAX_URL_LEN: usize = 4096;

/// How much of a refused value an [`InvalidUrl`] keeps, in bytes: enough to recognize it in a
/// log.
const SHOWN_LEN: usize = 256;

/// Why a value is not a URL of the kind asked for.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidUrlReason {
    #[error("it is empty")]
    Empty,

    #[error("it is longer than {max} bytes")]
    TooLong { max: usize },

    #[error("it contains whitespace, a control character or a backslash")]
    ForbiddenCharacter,

    #[error("the scheme must be {expected}")]
    Scheme { expected: &'static str },

    #[error("it names no host")]
    NoHost,

    #[error("it carries credentials")]
    Credentials,

    #[error("it is malformed: {0}")]
    Malformed(url::ParseError),
}

/// A value refused by [`HttpsUrl::parse`] or [`HttpUrl::parse`].
///
/// `value` is cut to a few hundred bytes, and has its credentials removed when it carried some.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Invalid {kind} {value:?}: {reason}")]
pub struct InvalidUrl {
    pub kind: &'static str,
    pub value: String,
    pub reason: InvalidUrlReason,
}

/// The schemes one kind of URL may have, each as written before `//`.
struct Schemes {
    allowed: &'static [&'static str],
    /// For the message: `https`, `http or https`.
    expected: &'static str,
}

const HTTPS: Schemes = Schemes {
    allowed: &["https"],
    expected: "https",
};

const HTTP_OR_HTTPS: Schemes = Schemes {
    allowed: &["http", "https"],
    expected: "http or https",
};

/// `value` as an [`InvalidUrl`] keeps it: without what precedes an `@` in its authority, which
/// may be a password, and cut to [`SHOWN_LEN`]. Done on the text, as the value may not parse.
fn shown(value: &str) -> String {
    let after_scheme = value.find(':').map_or(0, |colon| colon + 1);
    let authority_start = value[after_scheme..]
        .find(|c| c != '/' && c != '\\')
        .map_or(value.len(), |offset| after_scheme + offset);
    let authority = &value[authority_start..];
    let authority_end = authority
        .find(['/', '\\', '?', '#'])
        .unwrap_or(authority.len());

    let redacted;
    let value = match authority[..authority_end].rfind('@') {
        Some(at) => {
            redacted = format!("{}{}", &value[..authority_start], &authority[at + 1..]);
            redacted.as_str()
        }
        None => value,
    };

    if value.len() <= SHOWN_LEN {
        return value.to_owned();
    }
    let mut end = SHOWN_LEN;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &value[..end])
}

fn parse_web_url(value: &str, schemes: &Schemes, kind: &'static str) -> Result<Url, InvalidUrl> {
    let refuse = |reason| InvalidUrl {
        kind,
        value: shown(value),
        reason,
    };

    if value.is_empty() {
        return Err(refuse(InvalidUrlReason::Empty));
    }
    if value.len() > MAX_URL_LEN {
        return Err(refuse(InvalidUrlReason::TooLong { max: MAX_URL_LEN }));
    }
    if value
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
    {
        return Err(refuse(InvalidUrlReason::ForbiddenCharacter));
    }

    // Checked on the text, before parsing: the parser would take `https:host` or `https:///host`
    // for `https://host/`.
    let authority = schemes.allowed.iter().find_map(|scheme| {
        let prefix = value.get(..scheme.len() + 3)?;
        let (name, slashes) = prefix.split_at(scheme.len());
        (name.eq_ignore_ascii_case(scheme) && slashes == "://").then(|| &value[prefix.len()..])
    });
    let Some(authority) = authority else {
        return Err(refuse(InvalidUrlReason::Scheme {
            expected: schemes.expected,
        }));
    };
    if authority.is_empty() || authority.starts_with('/') {
        return Err(refuse(InvalidUrlReason::NoHost));
    }

    let url = Url::parse(value).map_err(|err| {
        refuse(match err {
            url::ParseError::EmptyHost => InvalidUrlReason::NoHost,
            err => InvalidUrlReason::Malformed(err),
        })
    })?;

    if url.host_str().is_none_or(str::is_empty) {
        return Err(refuse(InvalidUrlReason::NoHost));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(refuse(InvalidUrlReason::Credentials));
    }

    Ok(url)
}

macro_rules! define_web_url {
    ($(#[$meta:meta])* $name:ident, $schemes:expr, $kind:literal) => {
        $(#[$meta])*
        ///
        /// Kept as the parser writes it back: the scheme and host lowercased, an empty path as
        /// `/`. That is what [`Self::as_str`] gives and what it is serialized as.
        #[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(Url);

        impl $name {
            /// `value`, if it is a URL of this kind.
            pub fn parse(value: &str) -> Result<Self, InvalidUrl> {
                parse_web_url(value, &$schemes, $kind).map(Self)
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            pub fn as_url(&self) -> &Url {
                &self.0
            }

            /// The host, as the request will name it; never empty.
            pub fn host_str(&self) -> &str {
                self.0.host_str().unwrap_or_default()
            }

            /// Whether the host may be on the public internet: not `localhost` nor an address
            /// literal of a private, local or reserved network. Only what is written in the URL
            /// is judged; a name that resolves to a private address passes.
            pub fn names_public_host(&self) -> bool {
                match self.0.host() {
                    Some(Host::Domain(domain)) => !is_localhost(domain),
                    Some(Host::Ipv4(ip)) => !is_blocked_ip(IpAddr::V4(ip)),
                    Some(Host::Ipv6(ip)) => !is_blocked_ip(IpAddr::V6(ip)),
                    None => false,
                }
            }
        }

        impl TryFrom<&str> for $name {
            type Error = InvalidUrl;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::parse(value)
            }
        }

        impl TryFrom<String> for $name {
            type Error = InvalidUrl;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::parse(&value)
            }
        }

        impl FromStr for $name {
            type Err = InvalidUrl;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::parse(value)
            }
        }

        impl From<$name> for String {
            fn from(url: $name) -> Self {
                url.0.into()
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($name)).field(&self.as_str()).finish()
            }
        }
    };
}

define_web_url!(
    /// An absolute `https` URL with a host and no credentials: what the launcher fetches on
    /// someone else's word (a download of a plan or a pack, an icon).
    HttpsUrl,
    HTTPS,
    "https URL"
);

define_web_url!(
    /// An absolute `http` or `https` URL with a host and no credentials: a page the launcher only
    /// links to (a project's or a version's page), never fetches.
    HttpUrl,
    HTTP_OR_HTTPS,
    "web URL"
);

impl From<HttpsUrl> for HttpUrl {
    fn from(url: HttpsUrl) -> Self {
        Self(url.0)
    }
}

/// A link or an icon a record is shown with, from a source that writes "none" as an empty
/// string: `None` when it is absent or empty, and when it is not a URL of kind `U`, which is
/// logged as the `what` of the record. Nothing depends on it, so a bad one costs the value and
/// not the record; a URL to download is parsed strictly instead.
pub fn display_url<U>(what: &str, value: Option<String>) -> Option<U>
where
    U: for<'a> TryFrom<&'a str, Error = InvalidUrl>,
{
    let value = value.filter(|value| !value.is_empty())?;
    U::try_from(value.as_str())
        .inspect_err(|err| tracing::warn!("Dropping the {what}: {err}"))
        .ok()
}

#[cfg(test)]
impl HttpsUrl {
    /// A URL of a test server on this machine, which serves plain `http`.
    ///
    /// # Panics
    ///
    /// When `value` is not an `http` URL of a loopback address.
    pub(crate) fn from_test_server(value: &str) -> Self {
        let url = HttpUrl::parse(value).expect("a test server URL should parse");
        let loopback = match url.as_url().host() {
            Some(Host::Ipv4(ip)) => ip.is_loopback(),
            Some(Host::Ipv6(ip)) => ip.is_loopback(),
            _ => false,
        };
        assert!(loopback, "{value} is not on a loopback address");
        Self(url.0)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::modrinth_cdn(
        "https://cdn.modrinth.com/data/AANobbMI/versions/IZskON6d/sodium-0.6.13.jar"
    )]
    #[case::query_and_fragment("https://example.com/a.jar?token=x&y=1#frag")]
    #[case::port("https://example.com:8443/a.jar")]
    #[case::ipv4_literal("https://8.8.8.8/a.jar")]
    #[case::ipv6_literal("https://[2606:4700:4700::1111]/a.jar")]
    #[case::percent_encoded("https://example.com/a%20b.jar")]
    fn should_accept_an_https_url_when_it_is_well_formed(#[case] value: &str) {
        let url = HttpsUrl::parse(value).unwrap();

        assert_eq!(url.as_str(), value);
        assert_eq!(url.to_string(), value);
    }

    #[test]
    fn should_keep_the_url_as_the_parser_writes_it_when_it_is_not_canonical() {
        let url = HttpsUrl::parse("HTTPS://Example.COM").unwrap();

        assert_eq!(url.as_str(), "https://example.com/");
        assert_eq!(url.host_str(), "example.com");
    }

    #[rstest]
    #[case::empty("", InvalidUrlReason::Empty)]
    #[case::http("http://example.com/a.jar", InvalidUrlReason::Scheme { expected: "https" })]
    #[case::ftp("ftp://example.com/a.jar", InvalidUrlReason::Scheme { expected: "https" })]
    #[case::javascript("javascript:alert(1)", InvalidUrlReason::Scheme { expected: "https" })]
    #[case::data("data:image/png;base64,AAAA", InvalidUrlReason::Scheme { expected: "https" })]
    #[case::file("file:///etc/passwd", InvalidUrlReason::Scheme { expected: "https" })]
    #[case::relative("/cache/icon.png", InvalidUrlReason::Scheme { expected: "https" })]
    #[case::no_slashes("https:example.com/a.jar", InvalidUrlReason::Scheme { expected: "https" })]
    #[case::one_slash("https:/example.com/a.jar", InvalidUrlReason::Scheme { expected: "https" })]
    #[case::scheme_only("https://", InvalidUrlReason::NoHost)]
    #[case::three_slashes("https:///example.com/a.jar", InvalidUrlReason::NoHost)]
    #[case::port_only("https://:443/a.jar", InvalidUrlReason::NoHost)]
    #[case::leading_space(" https://example.com/", InvalidUrlReason::ForbiddenCharacter)]
    #[case::trailing_newline("https://example.com/\n", InvalidUrlReason::ForbiddenCharacter)]
    #[case::inner_tab("https://exa\tmple.com/", InvalidUrlReason::ForbiddenCharacter)]
    #[case::inner_space("https://example.com/a b.jar", InvalidUrlReason::ForbiddenCharacter)]
    #[case::nul("https://example.com/\0", InvalidUrlReason::ForbiddenCharacter)]
    #[case::backslash("https:\\\\example.com\\a.jar", InvalidUrlReason::ForbiddenCharacter)]
    #[case::bad_port(
        "https://example.com:99999/",
        InvalidUrlReason::Malformed(url::ParseError::InvalidPort)
    )]
    fn should_refuse_a_value_when_it_is_not_an_https_url(
        #[case] value: &str,
        #[case] reason: InvalidUrlReason,
    ) {
        let err = HttpsUrl::parse(value).unwrap_err();

        assert_eq!(err.reason, reason, "{value:?}");
        assert_eq!(err.kind, "https URL");
        assert_eq!(err.value, value);
    }

    #[rstest]
    #[case::user_and_password("https://user:secret@example.com/a.jar")]
    #[case::user("https://user@example.com/a.jar")]
    #[case::password("https://:secret@example.com/a.jar")]
    #[case::confusable_host("https://cdn.modrinth.com@evil.example/a.jar")]
    fn should_refuse_a_url_without_echoing_its_credentials_when_it_carries_some(
        #[case] value: &str,
    ) {
        let err = HttpsUrl::parse(value).unwrap_err();

        assert_eq!(err.reason, InvalidUrlReason::Credentials);
        assert!(!err.value.contains("secret"), "{err}");
        assert!(!err.value.contains('@'), "{err}");
        assert!(!err.to_string().contains("secret"), "{err}");
        assert!(err.value.ends_with("/a.jar"), "{err}");
    }

    /// Refused for another reason first, the value still loses what may be a password.
    #[rstest]
    #[case::http("http://user:secret@example.com/a.jar")]
    #[case::inner_space("https://user:secret@exa mple.com/a.jar")]
    #[case::one_slash("https:/user:secret@example.com/a.jar")]
    #[case::backslashes("https:\\\\user:secret@example.com\\a.jar")]
    fn should_not_echo_credentials_when_a_url_is_refused_for_another_reason(#[case] value: &str) {
        let err = HttpsUrl::parse(value).unwrap_err();

        assert_ne!(err.reason, InvalidUrlReason::Credentials);
        assert!(!err.to_string().contains("secret"), "{err}");
    }

    #[test]
    fn should_refuse_a_url_when_it_is_longer_than_the_limit() {
        let base = "https://example.com/";
        let fits = format!("{base}{}", "a".repeat(MAX_URL_LEN - base.len()));
        let overlong = format!("{fits}a");

        assert!(HttpsUrl::parse(&fits).is_ok());
        let err = HttpsUrl::parse(&overlong).unwrap_err();
        assert_eq!(err.reason, InvalidUrlReason::TooLong { max: MAX_URL_LEN });
        assert!(err.value.len() < 300, "the value shown is cut");
    }

    #[rstest]
    #[case::https("https://modrinth.com/mod/sodium")]
    #[case::http("http://example.com/mod")]
    fn should_accept_a_web_url_when_it_is_http_or_https(#[case] value: &str) {
        assert_eq!(HttpUrl::parse(value).unwrap().as_str(), value);
    }

    #[rstest]
    #[case::javascript("javascript:alert(1)")]
    #[case::data("data:text/html,<script>alert(1)</script>")]
    #[case::file("file:///C:/Users/me/secret.png")]
    #[case::ftp("ftp://example.com/")]
    #[case::mailto("mailto:me@example.com")]
    fn should_refuse_a_web_url_when_its_scheme_is_not_http_or_https(#[case] value: &str) {
        let err = HttpUrl::parse(value).unwrap_err();

        assert_eq!(
            err.reason,
            InvalidUrlReason::Scheme {
                expected: "http or https"
            }
        );
        assert_eq!(err.kind, "web URL");
    }

    #[rstest]
    #[case::domain("https://cdn.modrinth.com/icon.png", true)]
    #[case::public_ipv4("https://8.8.8.8/icon.png", true)]
    #[case::public_ipv6("https://[2606:4700:4700::1111]/icon.png", true)]
    #[case::localhost("https://localhost/icon.png", false)]
    #[case::localhost_subdomain("https://api.localhost./icon.png", false)]
    #[case::loopback("https://127.0.0.1/icon.png", false)]
    #[case::decimal_loopback("https://2130706433/icon.png", false)]
    #[case::private("https://192.168.0.1/icon.png", false)]
    #[case::cgnat("https://100.64.0.1/icon.png", false)]
    #[case::metadata_service("https://169.254.169.254/latest", false)]
    #[case::ipv6_loopback("https://[::1]/icon.png", false)]
    #[case::ipv6_unique_local("https://[fd00::1]/icon.png", false)]
    #[case::ipv4_mapped_private("https://[::ffff:10.0.0.1]/icon.png", false)]
    fn should_tell_a_public_host_when_asked(#[case] value: &str, #[case] public: bool) {
        assert_eq!(
            HttpsUrl::parse(value).unwrap().names_public_host(),
            public,
            "{value}"
        );
    }

    #[test]
    fn should_round_trip_as_a_plain_string_when_serialized() {
        let url: HttpsUrl = serde_json::from_str("\"https://example.com/a.jar\"").unwrap();

        assert_eq!(
            serde_json::to_string(&url).unwrap(),
            "\"https://example.com/a.jar\""
        );
        assert!(serde_json::from_str::<HttpsUrl>("\"http://example.com/a.jar\"").is_err());
        assert!(serde_json::from_str::<HttpUrl>("\"javascript:alert(1)\"").is_err());
    }

    #[test]
    fn should_keep_a_display_url_when_it_is_of_the_kind_asked() {
        assert_eq!(
            display_url::<HttpsUrl>("icon", Some("https://example.com/a.png".into())),
            Some(HttpsUrl::parse("https://example.com/a.png").unwrap())
        );
        assert_eq!(
            display_url::<HttpUrl>("page", Some("http://example.com/".into())),
            Some(HttpUrl::parse("http://example.com/").unwrap())
        );
    }

    #[rstest]
    #[case::absent(None)]
    #[case::empty(Some(""))]
    #[case::http(Some("http://example.com/a.png"))]
    #[case::javascript(Some("javascript:alert(1)"))]
    #[case::path(Some("/cache/icon.png"))]
    fn should_drop_a_display_url_when_it_is_absent_empty_or_not_of_the_kind_asked(
        #[case] value: Option<&str>,
    ) {
        assert_eq!(display_url::<HttpsUrl>("icon", value.map(Into::into)), None);
    }

    #[test]
    fn should_take_an_https_url_for_a_web_url_when_converted() {
        let url = HttpUrl::from(HttpsUrl::parse("https://example.com/").unwrap());

        assert_eq!(url.as_str(), "https://example.com/");
    }
}
