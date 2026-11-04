use crate::features::instance::InvalidContentId;

/// A stored content or version id as a typed one. One that fails the check can never be a
/// provider's id, so it is dropped rather than failing the read of the whole record; the stored
/// value is left as it is.
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
