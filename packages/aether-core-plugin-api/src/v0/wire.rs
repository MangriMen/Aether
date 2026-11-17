//! How the DTOs of this crate cross the plugin boundary.
//!
//! Every structured value, in both directions, is msgpack with structs written as maps keyed by
//! field name, byte fields as `bin`, and enums in their serde (JSON) shape. Arrays are not part of
//! the contract: they leave the field order to carry the meaning, and an adjacently tagged enum
//! (`HostError`, `HostResult`) cannot be read back from one.

use serde::{Serialize, de::DeserializeOwned};

pub use rmp_serde::{decode::Error as DecodeError, encode::Error as EncodeError};

/// Whether `bytes` start a msgpack map: what every struct on the boundary must be.
#[must_use]
pub fn is_map(bytes: &[u8]) -> bool {
    matches!(bytes.first(), Some(0x80..=0x8f | 0xde | 0xdf))
}

/// The bytes to hand to the other side.
///
/// # Errors
/// When `value` cannot be represented in msgpack.
pub fn encode<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, EncodeError> {
    rmp_serde::to_vec_named(value)
}

/// A value from the bytes the other side wrote. Unknown keys are ignored and a missing key with a
/// `#[serde(default)]` takes its default, so either side may add fields.
///
/// # Errors
/// When `bytes` are not a msgpack value of this shape.
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, DecodeError> {
    rmp_serde::from_slice(bytes)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    use crate::v0::{HostError, HostResult};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Old {
        a: u32,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct New {
        a: u32,
        #[serde(default)]
        b: bool,
    }

    #[test]
    fn a_missing_key_reads_as_its_default() {
        let bytes = encode(&Old { a: 1 }).unwrap();

        assert_eq!(decode::<New>(&bytes).unwrap(), New { a: 1, b: false });
    }

    #[test]
    fn an_unknown_key_is_ignored() {
        let bytes = encode(&New { a: 1, b: true }).unwrap();

        assert_eq!(decode::<Old>(&bytes).unwrap(), Old { a: 1 });
    }

    #[test]
    fn field_names_are_camel_case_in_structs_and_in_error_variants() {
        use crate::v0::{CommandDto, CommandRefusalDto, OutputDto};

        let command = serde_json::to_value(CommandDto {
            program: "java".to_owned(),
            args: vec![],
            current_dir: Some("#/cache".to_owned()),
        })
        .unwrap();
        let output = serde_json::to_value(OutputDto {
            exit_code: Some(0),
            stdout: vec![],
            stderr: vec![],
            stdout_truncated: false,
            stderr_truncated: false,
        })
        .unwrap();
        let refusal = serde_json::to_value(HostError::CommandNotApproved {
            message: "m".to_owned(),
            reason: CommandRefusalDto::Denied,
        })
        .unwrap();

        assert!(command.get("currentDir").is_some());
        assert!(output.get("stdoutTruncated").is_some());
        assert_eq!(refusal["type"], "commandNotApproved");
        assert_eq!(refusal["data"]["reason"], "denied");
    }

    #[test]
    fn a_result_is_a_map_in_both_cases() {
        let ok = encode(&HostResult::Ok(())).unwrap();
        let err = encode(&HostResult::<()>::Err(HostError::Internal {
            message: "disk".to_owned(),
        }))
        .unwrap();

        assert!(is_map(&ok) && is_map(&err));
        assert!(matches!(
            decode::<HostResult<()>>(&ok),
            Ok(HostResult::Ok(()))
        ));
        assert!(matches!(
            decode::<HostResult<()>>(&err),
            Ok(HostResult::Err(HostError::Internal { .. }))
        ));
    }
}

#[cfg(test)]
mod forward_compatibility_tests {
    use super::*;
    use crate::v0::{CommandRefusalDto, ContentTypeDto, InstanceInstallStageDto, ModLoaderDto};

    /// A name a newer launcher has and this version does not: the value is read as `Unknown`, not
    /// refused, so the rest of the call can still be read.
    fn read_unknown<T: DeserializeOwned>() -> T {
        decode(&encode("somethingNew").unwrap()).unwrap()
    }

    #[test]
    fn a_new_value_of_a_plain_enum_is_read_as_unknown() {
        assert_eq!(read_unknown::<ModLoaderDto>(), ModLoaderDto::Unknown);
        assert_eq!(read_unknown::<ContentTypeDto>(), ContentTypeDto::Unknown);
        assert_eq!(
            read_unknown::<CommandRefusalDto>(),
            CommandRefusalDto::Unknown
        );
        assert_eq!(
            read_unknown::<InstanceInstallStageDto>(),
            InstanceInstallStageDto::Unknown
        );
    }

    #[test]
    fn an_http_error_crosses_with_its_kind_and_a_new_kind_reads_as_unknown() {
        use serde_json::json;

        use crate::v0::{HostError, HttpErrorDto};

        let refused = HostError::Http {
            message: "Host `cdn.example` is not listed in the plugin's allowed_hosts".to_owned(),
            error: HttpErrorDto::HostNotAllowed {
                host: "cdn.example".to_owned(),
            },
        };
        let mismatch = HostError::Http {
            message: "m".to_owned(),
            error: HttpErrorDto::HashMismatch,
        };
        let http = |error: serde_json::Value| {
            encode(&json!({"type": "http", "data": {"message": "m", "error": error}})).unwrap()
        };

        assert_eq!(
            decode::<HostError>(&encode(&refused).unwrap()).unwrap(),
            refused
        );
        assert_eq!(
            decode::<HostError>(&encode(&mismatch).unwrap()).unwrap(),
            mismatch
        );
        assert_eq!(
            decode::<HostError>(&http(
                json!({"type": "somethingNew", "data": {"host": "x", "port": 1}})
            ))
            .unwrap(),
            HostError::Http {
                message: "m".to_owned(),
                error: HttpErrorDto::Unknown,
            }
        );
        assert_eq!(
            decode::<HostError>(&http(json!({"type": "somethingNew"}))).unwrap(),
            HostError::Http {
                message: "m".to_owned(),
                error: HttpErrorDto::Unknown,
            }
        );
        // A kind this version knows, with what it needs missing, is an error, not `unknown`.
        assert!(decode::<HostError>(&http(json!({"type": "hostNotAllowed", "data": {}}))).is_err());
    }

    #[test]
    fn a_new_value_is_read_inside_a_struct_without_failing_the_struct() {
        #[derive(Serialize, serde::Deserialize)]
        struct New {
            loader: &'static str,
            name: &'static str,
        }
        #[derive(serde::Deserialize)]
        struct Old {
            loader: ModLoaderDto,
            name: String,
        }

        let old: Old = decode(
            &encode(&New {
                loader: "somethingNew",
                name: "x",
            })
            .unwrap(),
        )
        .unwrap();

        assert_eq!(old.loader, ModLoaderDto::Unknown);
        assert_eq!(old.name, "x");
    }
}
