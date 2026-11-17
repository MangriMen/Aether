use aether_core_plugin_api::v0::{HostError, HostResult, wire};
use extism_convert::{FromBytesOwned, ToBytes};
use serde::{Serialize, de::DeserializeOwned};

/// A value as it crosses the plugin boundary: msgpack with named fields, see
/// [`aether_core_plugin_api::v0::wire`]. The one codec for every structured argument and result;
/// `extism_convert::Msgpack` writes structs as arrays and must not be used on the boundary.
#[derive(Debug)]
pub(crate) struct Wire<T>(pub T);

impl<T: Serialize> ToBytes<'_> for Wire<T> {
    type Bytes = Vec<u8>;

    fn to_bytes(&self) -> Result<Self::Bytes, extism_convert::Error> {
        Ok(wire::encode(&self.0)?)
    }
}

impl<T: DeserializeOwned> FromBytesOwned for Wire<T> {
    fn from_bytes_owned(data: &[u8]) -> Result<Self, extism_convert::Error> {
        Ok(Self(wire::decode(data)?))
    }
}

/// What a host function returns to the plugin.
pub(crate) type HostReply<T> = Wire<HostResult<T>>;

/// The argument of a host function as the plugin sent it: decoded, or why it is not. Reading it
/// never fails, so that a wrong shape (a missing field, a renamed key, an unknown value) can be
/// answered with `invalidArgument` instead of trapping the plugin's call, as [`Wire`] would.
#[derive(Debug)]
pub(crate) struct HostArg<T>(Result<T, wire::DecodeError>);

impl<T: DeserializeOwned> FromBytesOwned for HostArg<T> {
    fn from_bytes_owned(data: &[u8]) -> Result<Self, extism_convert::Error> {
        Ok(Self(wire::decode(data)))
    }
}

impl<T> HostArg<T> {
    /// The argument, or the answer to give the plugin when it did not decode. `function` names
    /// the host function in the message, next to the decoder's text (which holds no host path).
    pub(crate) fn decode<R>(self, function: &str) -> Result<Wire<T>, HostReply<R>> {
        self.0.map(Wire).map_err(|err| {
            Wire(HostResult::Err(HostError::InvalidArgument {
                message: format!("The argument of `{function}` is not valid: {err}"),
            }))
        })
    }
}

/// The answer to give the plugin for what a host function's handler returned.
pub(crate) fn to_extism_res<T>(res: crate::Result<T>) -> HostReply<T> {
    Wire(match res {
        Ok(v) => HostResult::Ok(v),
        Err(e) => HostResult::Err(HostError::from(e.raw.as_ref())),
    })
}

#[cfg(test)]
mod tests {
    use aether_core_plugin_api::v0::{CommandRefusalDto, HostError, HostResult, wire};
    use extism_convert::ToBytes;

    use super::to_extism_res;
    use crate::{
        ErrorKind,
        features::{
            instance::{InstanceError, PluginId},
            plugins::{CommandApprovalRefusal, PluginError},
        },
    };

    /// What the plugin sees: the error after the msgpack the host function returns.
    fn through_msgpack(kind: ErrorKind) -> HostError {
        let res: crate::Result<()> = Err(kind.as_error());
        let bytes = to_extism_res(res).to_bytes().expect("encodes");
        assert!(
            wire::is_map(bytes.as_ref()),
            "the result is not a msgpack map"
        );
        let decoded: HostResult<()> = wire::decode(bytes.as_ref()).expect("decodes");
        match decoded {
            HostResult::Err(error) => error,
            HostResult::Ok(()) => panic!("an error became Ok"),
        }
    }

    /// The bytes of a file or a response cross the plugin boundary as a msgpack `bin`: as an
    /// array of numbers the plugin would spend fuel on every byte (a 1.5 MiB jar used up the
    /// whole budget of an `import` call).
    #[test]
    fn should_hand_bytes_over_as_a_msgpack_bin_when_encoding_a_result() {
        let body = vec![7_u8; 4096];
        let response = aether_core_plugin_api::v0::HttpResponseDto {
            status: 200,
            headers: std::collections::HashMap::new(),
            body: body.clone(),
        };

        let bytes = wire::encode(&response).expect("encodes");
        assert!(wire::is_map(&bytes));

        assert!(
            bytes.len() < body.len() + 64,
            "{} bytes for a {}-byte body: the body is not a bin",
            bytes.len(),
            body.len()
        );
        let decoded: aether_core_plugin_api::v0::HttpResponseDto =
            wire::decode(&bytes).expect("decodes");
        assert_eq!(decoded.body, body);
    }

    #[test]
    fn should_hand_the_kind_and_message_over_when_a_host_function_fails() {
        let host = through_msgpack(
            InstanceError::FileTooLarge {
                path: "a.jar".to_owned(),
                size: u64::MAX,
                limit: 7,
            }
            .into(),
        );

        assert_eq!(
            host,
            HostError::TooLarge {
                message: format!(
                    "File \"a.jar\" is {} bytes, over the 7 byte limit",
                    u64::MAX
                ),
            }
        );
    }

    #[test]
    fn should_hand_a_refused_command_over_with_its_reason() {
        let host = through_msgpack(
            PluginError::CommandNotApproved {
                plugin_id: PluginId::from_static("packwiz"),
                reason: CommandApprovalRefusal::JarChanged,
            }
            .into(),
        );

        assert!(
            matches!(
                host,
                HostError::CommandNotApproved {
                    reason: CommandRefusalDto::JarChanged,
                    ..
                }
            ),
            "{host:?}"
        );
    }
}
