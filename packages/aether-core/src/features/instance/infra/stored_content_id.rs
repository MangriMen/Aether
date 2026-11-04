use crate::features::instance::{ContentVersionId, InvalidContentId};

/// A stored content or version id as a typed one. One that fails the check can never be a
/// provider's id, so it is dropped rather than failing the read of the whole record; the stored
/// value is left as it is. What the caller drops with it differs: a pack file's update entry is
/// dropped as a whole, a pack info is dropped when its `modpack_id` is invalid (its version label
/// is free-form and is never checked), an update entry keeps its content id when only its
/// version is invalid ([`parse_stored_version`]).
///
/// Logged at `debug`: the stored value stays, so a warning would repeat on every listing.
pub(super) fn parse_stored_content_id<T>(owner: &str, stored: String) -> Option<T>
where
    T: TryFrom<String, Error = InvalidContentId>,
{
    T::try_from(stored)
        .inspect_err(|err| tracing::debug!("Ignoring a stored id of {owner}: {err}"))
        .ok()
}

/// A stored version id, or `None` for an empty one (old records have no version) or one that
/// is not a valid id.
pub(super) fn parse_stored_version(owner: &str, stored: String) -> Option<ContentVersionId> {
    if stored.is_empty() {
        return None;
    }
    parse_stored_content_id(owner, stored)
}
