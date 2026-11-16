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
    use crate::v0::{AuthErrorDto, HostError, HostResult};

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
    fn a_result_is_a_map_in_both_cases() {
        let ok = encode(&HostResult::Ok(())).unwrap();
        let err = encode(&HostResult::<()>::Err(HostError::AuthError(
            AuthErrorDto::TokenExpired,
        )))
        .unwrap();

        assert!(is_map(&ok) && is_map(&err));
        assert!(matches!(
            decode::<HostResult<()>>(&ok),
            Ok(HostResult::Ok(()))
        ));
        assert!(matches!(
            decode::<HostResult<()>>(&err),
            Ok(HostResult::Err(HostError::AuthError(
                AuthErrorDto::TokenExpired
            )))
        ));
    }
}
