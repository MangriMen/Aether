//! Schema fragments the derive cannot produce.

use schemars::{Schema, SchemaGenerator, json_schema};

/// A byte string: on the wire a msgpack `bin`, so no array of numbers. A JSON validator sees a
/// string (a `bin` has no JSON form); `x-msgpack` says what it really is.
pub(crate) fn bytes(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "contentEncoding": "binary",
        "x-msgpack": "bin"
    })
}
