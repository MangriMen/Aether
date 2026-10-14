use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

/// Parameters of the `write_instance_file` host function — the only way a plugin may put a
/// file inside an instance directory.
///
/// The path is **not** a WASI path: `/instances/<id>/...` and `/mnt/d/...` forms are refused.
/// A plugin names the instance by id and the file by a path relative to that instance's root,
/// and the host decides where that lands on disk.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WriteInstanceFileParamsDto {
    /// Id of an existing instance. An unknown id is an error, not a directory the host creates.
    pub instance_id: String,
    /// Path relative to the instance root, `/`-separated. Must be made of plain name segments:
    /// no leading separator, no drive letter, no `.` or `..`.
    pub relative_path: String,
    /// File contents. The whole file is replaced; the host caps the size. Travels as a msgpack
    /// `bin` as the body of `HttpResponseDto` does.
    #[serde_as(as = "Bytes")]
    pub bytes: Vec<u8>,
}

/// What `read_instance_file` returns: the file contents as a msgpack `bin`.
///
/// A newtype, so that the bytes can be encoded as `bin` — a bare `Vec<u8>` result would be an
/// array of numbers.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstanceFileBytesDto(#[serde_as(as = "Bytes")] pub Vec<u8>);

/// Parameters of the `read_instance_file` host function.
///
/// Same path rules as [`WriteInstanceFileParamsDto`].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadInstanceFileParamsDto {
    /// Id of an existing instance.
    pub instance_id: String,
    /// Path relative to the instance root, `/`-separated.
    pub relative_path: String,
}
