use crate::features::instance::{ContentPath, InstanceId};

/// A stored content path as a [`ContentPath`]. One that fails the check can never name a file of
/// an instance, so it is dropped rather than failing the read of the whole pack; the stored row
/// or entry is left as it is.
///
/// Logged at `debug`: the stored value stays, so a warning would repeat on every listing.
pub(super) fn parse_stored_content_path(
    instance_id: &InstanceId,
    stored: &str,
) -> Option<ContentPath> {
    match ContentPath::parse(stored) {
        // Parsing normalizes the separators. A stored value that changes would be read back
        // under a path other than the one it is stored at.
        Ok(path) if path.as_str() == stored => Some(path),
        Ok(path) => {
            tracing::debug!(
                "Ignoring pack file \"{stored}\" of \"{instance_id}\": it is not in its normalized form \"{path}\""
            );
            None
        }
        Err(err) => {
            tracing::debug!(
                "Ignoring pack file \"{stored}\" of \"{instance_id}\": not a valid content path: {err}"
            );
            None
        }
    }
}
