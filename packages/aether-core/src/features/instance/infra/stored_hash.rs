use crate::{
    features::instance::parse_file_hash,
    shared::hash::{FileHash, InvalidFileHash},
};

/// The hash a pack file is stored with, or the reason it is not one: a record without a valid
/// hash cannot be told from any other file, so the caller drops the record.
pub(super) fn parse_stored_hash(
    file_name: &str,
    stored: &str,
) -> Result<FileHash, InvalidFileHash> {
    parse_file_hash(stored).inspect_err(|err| {
        tracing::warn!("Skipping pack file \"{file_name}\": invalid hash: {err}");
    })
}
