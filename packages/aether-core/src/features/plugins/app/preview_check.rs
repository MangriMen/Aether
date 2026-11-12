use std::path::Path;

use crate::features::plugins::{
    ExtractedPlugin, PluginContent, PluginManifestPreview, PluginProviderError,
};

const CAPABILITIES_FILE: &str = "capabilities.json";

/// Holds a downloaded archive to the manifest and capabilities its release published beside it,
/// which are what the preview shows: an archive that asks for more than that is refused, and so
/// is any other difference.
///
/// Both are read again at install time, so a publisher who swaps the release between the preview
/// and the install is not caught here. A release that publishes no manifest has nothing to hold
/// the archive to, and the preview showed no permissions either.
pub(crate) fn check_against_preview(
    extracted: &ExtractedPlugin,
    manifest: Option<&PluginManifestPreview>,
    capabilities: Option<&str>,
) -> Result<(), PluginProviderError> {
    let Some(shown) = manifest else {
        return Ok(());
    };
    let differs = |field| PluginProviderError::ArchiveDiffersFromPreview { field };

    if shown.id != extracted.plugin_id.as_str() {
        return Err(differs("id"));
    }
    let version = &extracted.manifest.metadata.version;
    if semver::Version::parse(shown.version.trim_start_matches('v'))
        .ok()
        .as_ref()
        != Some(version)
    {
        return Err(differs("version"));
    }
    if !same_set(
        &shown.allowed_hosts,
        &extracted.manifest.runtime.allowed_hosts,
    ) {
        return Err(differs("allowed_hosts"));
    }
    let declared = |list: &Option<Vec<String>>| list.clone().unwrap_or_default();
    if !same_set(
        &declared(&shown.host_functions),
        &declared(&extracted.manifest.api.host_functions),
    ) {
        return Err(differs("host_functions"));
    }
    if !same_capabilities(capabilities, archive_capabilities(extracted)) {
        return Err(differs("capabilities"));
    }
    Ok(())
}

/// The same entries, whatever their order.
fn same_set(a: &[String], b: &[String]) -> bool {
    let sorted = |list: &[String]| {
        let mut list = list.to_vec();
        list.sort();
        list
    };
    sorted(a) == sorted(b)
}

fn archive_capabilities(extracted: &ExtractedPlugin) -> Option<String> {
    let PluginContent::Filesystem { temp_dir } = &extracted.content;
    read_capabilities(temp_dir.path())
}

fn read_capabilities(dir: &Path) -> Option<String> {
    std::fs::read_to_string(dir.join(CAPABILITIES_FILE)).ok()
}

/// Both absent, or both the same JSON document: formatting is not a difference.
fn same_capabilities(shown: Option<&str>, archive: Option<String>) -> bool {
    match (shown, archive) {
        (None, None) => true,
        (Some(shown), Some(archive)) => {
            match (
                serde_json::from_str::<serde_json::Value>(shown),
                serde_json::from_str::<serde_json::Value>(&archive),
            ) {
                (Ok(shown), Ok(archive)) => shown == archive,
                _ => false,
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_treat_capabilities_as_the_same_when_only_the_formatting_differs() {
        assert!(same_capabilities(
            Some("{\"a\": 1}"),
            Some("{ \"a\":1 }\n".to_owned())
        ));
    }

    #[test]
    fn should_treat_capabilities_as_different_when_one_side_has_none() {
        assert!(!same_capabilities(None, Some("{}".to_owned())));
        assert!(!same_capabilities(Some("{}"), None));
        assert!(same_capabilities(None, None));
    }

    #[test]
    fn should_treat_capabilities_as_different_when_the_content_differs() {
        assert!(!same_capabilities(
            Some("{\"a\": 1}"),
            Some("{\"a\": 2}".to_owned())
        ));
    }

    #[test]
    fn should_compare_entries_without_regard_to_order() {
        let list = |items: &[&str]| items.iter().map(ToString::to_string).collect::<Vec<_>>();
        assert!(same_set(&list(&["a", "b"]), &list(&["b", "a"])));
        assert!(!same_set(&list(&["a"]), &list(&["a", "*"])));
    }
}
