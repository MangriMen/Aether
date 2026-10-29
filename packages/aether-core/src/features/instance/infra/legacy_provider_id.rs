//! Provider ids written by earlier versions of Aether, read as the ids they are now.
//!
//! Stored data is never rewritten in place: a legacy id is translated wherever it is read, and
//! the current id is what gets written back on the next save.

use std::str::FromStr;

use crate::features::instance::{
    InvalidId, ProviderId, ProviderIdError, infra::ModrinthContentProvider,
};

/// The built-in Modrinth provider in V1 pack files, before ids had a plugin part.
const MODRINTH_V1: &str = "modrinth";

/// The built-in Modrinth provider before the `aether` plugin id was reserved for it.
///
/// Its `:` is outside the id scheme, so it is recognised as a string, before any parsing.
const MODRINTH_CORE_PLUGIN_ID: &str = "core:modrinth";
const MODRINTH_CORE_CAPABILITY_ID: &str = "modrinth-content";

/// The string form of the built-in Modrinth provider before it was renamed.
fn legacy_modrinth_string() -> String {
    format!(
        "{MODRINTH_CORE_PLUGIN_ID}{}{MODRINTH_CORE_CAPABILITY_ID}",
        ProviderId::SEPARATOR
    )
}

/// Reads a provider id stored as its two parts, reading a legacy id as the id it is now.
pub(super) fn parse_stored_provider_id_parts(
    plugin_id: &str,
    capability_id: &str,
) -> Result<ProviderId, InvalidId> {
    if plugin_id == MODRINTH_CORE_PLUGIN_ID && capability_id == MODRINTH_CORE_CAPABILITY_ID {
        return Ok(ModrinthContentProvider::provider_id());
    }

    ProviderId::from_parts(plugin_id, capability_id)
}

/// Parses the string form of a stored provider id, reading a legacy id as the id it is now.
pub(super) fn parse_stored_provider_id(value: &str) -> Result<ProviderId, ProviderIdError> {
    if value == legacy_modrinth_string() {
        return Ok(ModrinthContentProvider::provider_id());
    }

    ProviderId::from_str(value)
}

/// [`parse_stored_provider_id`] for V1 pack files, which also wrote the built-in Modrinth
/// provider as a bare `modrinth`.
pub(super) fn parse_v1_provider_id(value: &str) -> Result<ProviderId, ProviderIdError> {
    if value == MODRINTH_V1 {
        return Ok(ModrinthContentProvider::provider_id());
    }

    parse_stored_provider_id(value)
}

/// The string form `id` was stored under before it was renamed, if it was.
///
/// For lookups by id in storage that is queried rather than read whole, where translating on
/// read cannot help.
pub(super) fn legacy_provider_id_string(id: &ProviderId) -> Option<String> {
    (*id == ModrinthContentProvider::provider_id()).then(legacy_modrinth_string)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn aether_modrinth() -> ProviderId {
        "aether#modrinth".parse().unwrap()
    }

    #[test]
    fn should_read_core_modrinth_as_aether_modrinth_when_given_the_old_id() {
        assert_eq!(
            parse_stored_provider_id("core:modrinth#modrinth-content").unwrap(),
            aether_modrinth()
        );
        assert_eq!(
            parse_stored_provider_id_parts("core:modrinth", "modrinth-content").unwrap(),
            aether_modrinth()
        );
    }

    #[test]
    fn should_read_v1_bare_modrinth_as_aether_modrinth_when_given_no_prefix() {
        assert_eq!(parse_v1_provider_id("modrinth").unwrap(), aether_modrinth());
        assert_eq!(
            parse_v1_provider_id("core:modrinth#modrinth-content").unwrap(),
            aether_modrinth()
        );
    }

    /// Outside V1 files `modrinth` is not an id at all — and `modrinth#modrinth` may belong to a
    /// plugin, so it must not be taken for the built-in provider.
    #[test]
    fn should_leave_other_ids_alone_when_they_are_not_modrinth() {
        assert!(parse_stored_provider_id("modrinth").is_err());
        assert_eq!(
            parse_stored_provider_id("modrinth#modrinth").unwrap(),
            "modrinth#modrinth".parse::<ProviderId>().unwrap()
        );
        assert_eq!(
            parse_stored_provider_id("packwiz#packwiz").unwrap(),
            "packwiz#packwiz".parse::<ProviderId>().unwrap()
        );
    }

    #[rstest]
    #[case("Bad Id#cap")]
    #[case("a:b#cap")]
    #[case("core:modrinth#other")]
    #[case("core:modrinth#modrinth-content#x")]
    fn should_refuse_a_stored_id_when_it_is_malformed_and_not_the_legacy_one(#[case] value: &str) {
        assert!(matches!(
            parse_stored_provider_id(value),
            Err(ProviderIdError::InvalidPart(_))
        ));
        assert!(parse_v1_provider_id(value).is_err());
    }

    #[test]
    fn should_refuse_stored_parts_when_they_are_malformed_and_not_the_legacy_ones() {
        assert!(parse_stored_provider_id_parts("a:b", "cap").is_err());
        assert!(parse_stored_provider_id_parts("core:modrinth", "other").is_err());
    }

    #[test]
    fn should_name_the_legacy_string_of_aether_modrinth_only_when_asked_for_other_ids() {
        assert_eq!(
            legacy_provider_id_string(&aether_modrinth()).as_deref(),
            Some("core:modrinth#modrinth-content")
        );
        assert_eq!(
            legacy_provider_id_string(&"packwiz#packwiz".parse().unwrap()),
            None
        );
    }
}
