//! What a Rust plugin needs to talk to the host: the wire encoding, reading what a host function
//! returned, and reporting a failure the host can tell apart.
//!
//! The DTOs, the failure envelope and its schema live in `aether-core-plugin-api`, which has no
//! Extism dependency; this crate is the Extism side of it.
//!
//! ```ignore
//! use extism_pdk::{FnResult, plugin_fn};
//!
//! #[host_fn]
//! extern "ExtismHost" {
//!     fn run_command(command: Wire<CommandDto>) -> HostReply<OutputDto>;
//! }
//!
//! // `#[plugin_fn]` wants `FnResult`; `?` turns a `pdk::Error` into it.
//! // `Wire` is `Msgpack` with named fields, which is what the host reads and writes; the
//! // plugin's own inputs and outputs (`#[plugin_fn] fn f(Wire(x): Wire<X>) -> FnResult<Wire<Y>>`)
//! // use it the same way.
//! fn run(command: CommandDto) -> pdk::Result<OutputDto> {
//!     Ok(unsafe { run_command(Wire(command)) }?.into_result()?)
//! }
//!
//! #[plugin_fn]
//! pub fn update() -> FnResult<()> {
//!     run(command)?;
//!     Ok(())
//! }
//! ```

use std::fmt;

use aether_core_plugin_api::v0::{
    HostError, HostResult, PluginFailureDto,
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
    /// A host function failed; the error goes back to the host unchanged.
    Host(HostError),
    /// The plugin's own failure.
    Message(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<HostError> for Error {
    fn from(value: HostError) -> Self {
        Self::Host(value)
    }
}

/// What Extism itself reports, such as a host function that could not be called.
impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Self::Message(format!("{value:#}"))
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
            Self::Host(error) => write!(f, "{error:?}"),
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
                Error::Host(error) => Some(error),
                Error::Message(_) => None,
            },
        };

        anyhow::Error::msg(failure.encode())
    }
}

/// The return type to declare a host function with: what the host wrote, decoded with [`Wire`].
#[derive(Debug)]
pub struct HostReply<T>(pub HostResult<T>);

impl<T> HostReply<T> {
    /// The value, or the host's error as an [`Error::Host`] for `?` to carry on.
    pub fn into_result(self) -> Result<T> {
        self.0.into_result().map_err(Error::Host)
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
    use aether_core_plugin_api::v0::{CommandRefusalDto, PluginErrorDto};

    use super::*;

    #[test]
    fn a_host_error_survives_the_trip_through_anyhow() {
        let host = HostError::PluginError(PluginErrorDto::CommandNotApproved {
            plugin_id: "p".to_owned(),
            reason: CommandRefusalDto::TimedOut,
        });

        let raw = format!("{:?}", anyhow::Error::from(Error::from(host)));
        let failure = PluginFailureDto::parse(&raw).expect("envelope");

        assert!(matches!(
            failure.host,
            Some(HostError::PluginError(PluginErrorDto::CommandNotApproved {
                reason: CommandRefusalDto::TimedOut,
                ..
            }))
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
        let err = wire::encode(&HostResult::<u32>::Err(HostError::CoreError(
            "x".to_owned(),
        )))
        .unwrap();

        let value = HostReply::<u32>::from_bytes_owned(&ok)
            .unwrap()
            .into_result();
        assert!(matches!(value, Ok(7)));

        let refused = HostReply::<u32>::from_bytes_owned(&err)
            .unwrap()
            .into_result();
        assert!(matches!(refused, Err(Error::Host(HostError::CoreError(_)))));
    }
}
