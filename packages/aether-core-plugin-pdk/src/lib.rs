//! What a Rust plugin needs to talk to the host: the wire encoding, reading what a host function
//! returned, and reporting a failure the host can tell apart.
//!
//! The DTOs, the failure envelope and its schema live in `aether-core-plugin-api`, which has no
//! Extism dependency; this crate is the Extism side of it, and re-exports it as [`api`], so that a
//! plugin needs only this one dependency (`aether_core_plugin_pdk::api::v0::CommandDto`).
//!
//! ```ignore
//! use aether_core_plugin_pdk::{
//!     self as pdk, ResultExt,
//!     api::v0::{CommandDto, OutputDto},
//! };
//! use extism_pdk::{FnResult, plugin_fn};
//!
//! // `#[plugin_fn]` wants `FnResult`; `?` turns a `pdk::Error` into it.
//! // `pdk::host` has one function per host function; a refusal of the host comes back as
//! // `pdk::Error::Host`. `Wire` is `Msgpack` with named fields, which is what the host reads and
//! // writes; the plugin's own inputs and outputs
//! // (`#[plugin_fn] fn f(Wire(x): Wire<X>) -> FnResult<Wire<Y>>`) use it.
//! // `context` puts the plugin's words before the host's message; the host error itself goes
//! // back to the host unchanged.
//! fn run(command: CommandDto) -> pdk::Result<OutputDto> {
//!     pdk::host::run_command(command).context("running the pack installer")
//! }
//!
//! #[plugin_fn]
//! pub fn update() -> FnResult<()> {
//!     run(command)?;
//!     Ok(())
//! }
//! ```

use std::fmt;

pub mod host;

pub use aether_core_plugin_api as api;
use aether_core_plugin_api::v0::{
    HostError, HostResult, PLUGIN_FAILURE_MARKER, PluginFailureDto,
    config::{CONFIG_API_VERSION, CONFIG_PLUGIN_ID},
    wire,
};
use extism_convert::{FromBytesOwned, ToBytes};
use serde::{Serialize, de::DeserializeOwned};

/// A value as it crosses the host boundary: msgpack with named fields, see
/// [`aether_core_plugin_api::v0::wire`]. Use it instead of `extism_pdk::Msgpack`, which writes
/// structs as arrays.
#[derive(Debug)]
pub struct Wire<T>(pub T);

impl<T: Serialize> ToBytes<'_> for Wire<T> {
    type Bytes = Vec<u8>;

    fn to_bytes(&self) -> std::result::Result<Self::Bytes, extism_convert::Error> {
        Ok(wire::encode(&self.0)?)
    }
}

impl<T: DeserializeOwned> FromBytesOwned for Wire<T> {
    fn from_bytes_owned(data: &[u8]) -> std::result::Result<Self, extism_convert::Error> {
        Ok(Self(wire::decode(data)?))
    }
}

#[derive(Debug)]
pub enum Error {
    /// A host function failed; `error` goes back to the host unchanged. `context` is what the
    /// plugin says it was doing ([`Error::context`]), put before the host's message in the
    /// envelope's `message`.
    Host {
        error: HostError,
        context: Option<String>,
    },
    /// The plugin's own failure.
    Message(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// The error with what the plugin was doing put in front of its message:
    /// `"downloading the pack: <message>"`. A host error stays the one the host sent.
    #[must_use]
    pub fn context(self, context: impl Into<String>) -> Self {
        let context = context.into();
        match self {
            Self::Host {
                error,
                context: None,
            } => Self::Host {
                error,
                context: Some(context),
            },
            Self::Host {
                error,
                context: Some(inner),
            } => Self::Host {
                error,
                context: Some(format!("{context}: {inner}")),
            },
            Self::Message(message) => Self::Message(format!("{context}: {message}")),
        }
    }

    /// The error a host function returned, if this is one.
    #[must_use]
    pub fn host_error(&self) -> Option<&HostError> {
        match self {
            Self::Host { error, .. } => Some(error),
            Self::Message(_) => None,
        }
    }
}

/// [`Error::context`] on a [`Result`]:
/// `pdk::host::download_to_cache(params).context("downloading the pack")?`.
pub trait ResultExt<T> {
    /// The error, if any, with `context` put in front of its message.
    ///
    /// # Errors
    ///
    /// The error of `self`, with `context` added.
    fn context(self, context: impl Into<String>) -> Result<T>;
}

impl<T> ResultExt<T> for Result<T> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|error| error.context(context))
    }
}

impl From<HostError> for Error {
    fn from(error: HostError) -> Self {
        Self::Host {
            error,
            context: None,
        }
    }
}

/// What Extism itself reports, such as a host function that could not be called, or an error of
/// this crate that went through `anyhow` (a helper returning `FnResult` called with `?`): its
/// failure envelope is read back, so that the host error is not buried in a second envelope
/// around the first, where the host would not find it. What `anyhow` put before the envelope is
/// kept as context.
impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        let text = format!("{value:#}");
        let Some(failure) = PluginFailureDto::parse(&text) else {
            return Self::Message(text);
        };

        let outer = text
            .find(PLUGIN_FAILURE_MARKER)
            .map(|at| text[..at].trim_end_matches([':', ' ']))
            .filter(|outer| !outer.is_empty());

        let error = match failure.host {
            Some(error) => {
                // The envelope's message is `<context>: <host message>`, or only the latter.
                let host_message = error.to_string();
                let context = if failure.message == host_message {
                    None
                } else {
                    Some(
                        failure
                            .message
                            .strip_suffix(&host_message)
                            .and_then(|context| context.strip_suffix(": "))
                            .unwrap_or(&failure.message)
                            .to_owned(),
                    )
                };
                Self::Host { error, context }
            }
            None => Self::Message(failure.message),
        };

        match outer {
            Some(outer) => error.context(outer),
            None => error,
        }
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Message(value.to_owned())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host {
                error,
                context: None,
            } => write!(f, "{error}"),
            Self::Host {
                error,
                context: Some(context),
            } => write!(f, "{context}: {error}"),
            Self::Message(message) => f.write_str(message),
        }
    }
}

// The error type of the plugin functions (`extism_pdk::Error`) is `anyhow::Error`; what it
// prints is what the host reads. Not an `impl std::error::Error`: that would collide with
// anyhow's own conversion.
impl From<Error> for anyhow::Error {
    fn from(value: Error) -> Self {
        let failure = PluginFailureDto {
            message: value.to_string(),
            host: match value {
                Error::Host { error, .. } => Some(error),
                Error::Message(_) => None,
            },
        };

        anyhow::Error::msg(failure.encode())
    }
}

/// The return type to declare a host function with: what the host wrote, decoded with [`Wire`].
/// The functions of [`host`] declare and read it already; it is for a plugin that declares its
/// own `#[host_fn]` imports.
#[derive(Debug)]
pub struct HostReply<T>(pub HostResult<T>);

impl<T> HostReply<T> {
    /// The value, or the host's error as an [`Error::Host`] for `?` to carry on.
    ///
    /// # Errors
    ///
    /// The [`HostError`] the host answered with.
    pub fn into_result(self) -> Result<T> {
        self.0.into_result().map_err(Error::from)
    }
}

impl<T: DeserializeOwned> FromBytesOwned for HostReply<T> {
    fn from_bytes_owned(data: &[u8]) -> std::result::Result<Self, extism_convert::Error> {
        Ok(Self(Wire::<HostResult<T>>::from_bytes_owned(data)?.0))
    }
}

/// The id the host knows this plugin by.
pub fn plugin_id() -> Result<String> {
    config_value(CONFIG_PLUGIN_ID)
}

/// The version of the plugin API the host implements (`MAJOR.MINOR.PATCH`).
pub fn host_api_version() -> Result<String> {
    config_value(CONFIG_API_VERSION)
}

fn config_value(key: &str) -> Result<String> {
    extism_pdk::config::get(key)?
        .ok_or_else(|| Error::Message(format!("the host did not set `{key}` in the plugin config")))
}

#[cfg(test)]
mod tests {
    use aether_core_plugin_api::v0::{CommandRefusalDto, HttpErrorDto};

    use super::*;

    #[test]
    fn a_host_error_survives_the_trip_through_anyhow() {
        let host = HostError::CommandNotApproved {
            reason: CommandRefusalDto::TimedOut,
            message: "nobody answered".to_owned(),
        };

        let raw = format!("{:?}", anyhow::Error::from(Error::from(host)));
        let failure = PluginFailureDto::parse(&raw).expect("envelope");

        assert_eq!(failure.message, "nobody answered");
        assert!(matches!(
            failure.host,
            Some(HostError::CommandNotApproved {
                reason: CommandRefusalDto::TimedOut,
                ..
            })
        ));
    }

    #[test]
    fn a_message_has_no_host_error() {
        let raw = format!("{:?}", anyhow::Error::from(Error::from("boom")));
        let failure = PluginFailureDto::parse(&raw).expect("envelope");

        assert_eq!(failure.message, "boom");
        assert!(failure.host.is_none());
    }

    #[test]
    fn a_reply_is_read_from_what_the_host_writes() {
        let ok = wire::encode(&HostResult::Ok(7_u32)).unwrap();
        let err = wire::encode(&HostResult::<u32>::Err(HostError::Internal {
            message: "x".to_owned(),
        }))
        .unwrap();

        let value = HostReply::<u32>::from_bytes_owned(&ok)
            .unwrap()
            .into_result();
        assert!(matches!(value, Ok(7)));

        let refused = HostReply::<u32>::from_bytes_owned(&err)
            .unwrap()
            .into_result();
        assert!(matches!(
            refused,
            Err(Error::Host {
                error: HostError::Internal { .. },
                context: None
            })
        ));
    }

    fn refused_host() -> HostError {
        HostError::Http {
            message: "Host `cdn.example` is not listed".to_owned(),
            error: HttpErrorDto::HostNotAllowed {
                host: "cdn.example".to_owned(),
            },
        }
    }

    /// What the host reads of an error the plugin's call ended with.
    fn envelope(error: Error) -> PluginFailureDto {
        PluginFailureDto::parse(&format!("{:?}", anyhow::Error::from(error))).expect("envelope")
    }

    /// A `pdk::Error` that went through a helper returning `FnResult` and back with `?`.
    #[test]
    fn a_host_error_survives_a_trip_through_anyhow_and_back() {
        let back = Error::from(anyhow::Error::from(Error::from(refused_host())));

        assert!(matches!(
            &back,
            Error::Host {
                error: HostError::Http {
                    error: HttpErrorDto::HostNotAllowed { host },
                    ..
                },
                context: None,
            } if host == "cdn.example"
        ));

        let failure = envelope(back);
        assert_eq!(failure.host, Some(refused_host()));
        assert_eq!(failure.message, "Host `cdn.example` is not listed");
    }

    #[test]
    fn the_context_of_a_host_error_survives_a_trip_through_anyhow_and_back() {
        let result: Result<()> = Err(Error::from(refused_host()));
        let error = result.context("downloading the pack").unwrap_err();
        let back = Error::from(anyhow::Error::from(error).context("updating"));

        assert_eq!(back.host_error(), Some(&refused_host()));

        let failure = envelope(back);
        assert_eq!(failure.host, Some(refused_host()));
        assert_eq!(
            failure.message,
            "updating: downloading the pack: Host `cdn.example` is not listed"
        );
    }

    #[test]
    fn a_plain_anyhow_error_stays_a_message() {
        let back = Error::from(anyhow::anyhow!("no such function").context("calling the host"));

        assert!(
            matches!(&back, Error::Message(message) if message == "calling the host: no such function"),
            "{back:?}"
        );
    }
}
