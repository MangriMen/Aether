//! What a new version of a plugin asks for that the installed one does not, and what it gives up,
//! so the user can see it before the update replaces the plugin.
//!
//! The core compares and classifies; a frontend only shows what it is told.

use std::collections::BTreeSet;

use super::host_function_risk::{HostFunctionClass, classify_host_function};
use super::{
    AsCapabilityMetadata, HostFunctionRisk, PluginCapabilities, PluginContentProviderCapability,
    PluginImporterCapability, PluginManifest, PluginUpdaterCapability, is_wide_allowed_host,
};

/// A host function one version declares and the other does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostFunctionChange {
    pub name: String,
    /// What the function lets the plugin do that the user is warned about; `None` for one that
    /// carries no warning or that the core does not know.
    pub risk: Option<HostFunctionRisk>,
}

/// An `allowed_hosts` entry one version has and the other does not, as the manifest spells it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedHostChange {
    pub entry: String,
    /// The entry admits far more than one site.
    pub wide: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapabilityKind {
    Importer,
    Updater,
    ContentProvider,
}

/// A capability, told apart by kind and id, that one version registers and the other does not,
/// or registers differently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityChange {
    pub kind: CapabilityKind,
    pub id: String,
    /// The name the version that has it gives it.
    pub name: String,
}

/// The difference between the installed version of a plugin and another one, from the second's
/// point of view: "added" is what the other version has and the installed one does not.
///
/// Every list is sorted, with no repeats. A manifest that does not declare its host functions is
/// compared as one that declares none.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PluginUpdateDiff {
    pub added_host_functions: Vec<HostFunctionChange>,
    pub removed_host_functions: Vec<HostFunctionChange>,
    /// Risks the other version carries that the installed one does not, in enum order. A risky
    /// function added beside one of the same risk adds none.
    pub added_risks: Vec<HostFunctionRisk>,
    pub added_allowed_hosts: Vec<AllowedHostChange>,
    pub removed_allowed_hosts: Vec<String>,
    pub added_capabilities: Vec<CapabilityChange>,
    pub removed_capabilities: Vec<CapabilityChange>,
    /// Capabilities both versions have under the same kind and id but declare differently, such
    /// as a content provider that now serves another platform.
    pub changed_capabilities: Vec<CapabilityChange>,
}

impl PluginUpdateDiff {
    pub fn between(
        installed: &PluginManifest,
        installed_capabilities: Option<&PluginCapabilities>,
        other: &PluginManifest,
        other_capabilities: Option<&PluginCapabilities>,
    ) -> Self {
        let installed_functions = host_functions(installed);
        let other_functions = host_functions(other);
        let installed_risks: BTreeSet<_> =
            installed.api.host_function_risks().into_iter().collect();
        let installed_hosts = allowed_hosts(installed);
        let other_hosts = allowed_hosts(other);
        let installed_caps = capabilities(installed_capabilities);
        let other_caps = capabilities(other_capabilities);

        Self {
            added_host_functions: function_changes(&other_functions, &installed_functions),
            removed_host_functions: function_changes(&installed_functions, &other_functions),
            added_risks: other
                .api
                .host_function_risks()
                .into_iter()
                .filter(|risk| !installed_risks.contains(risk))
                .collect(),
            added_allowed_hosts: other_hosts
                .difference(&installed_hosts)
                .map(|entry| AllowedHostChange {
                    entry: (*entry).to_owned(),
                    wide: is_wide_allowed_host(entry),
                })
                .collect(),
            removed_allowed_hosts: installed_hosts
                .difference(&other_hosts)
                .map(|entry| (*entry).to_owned())
                .collect(),
            added_capabilities: capability_changes(&other_caps, &installed_caps),
            removed_capabilities: capability_changes(&installed_caps, &other_caps),
            changed_capabilities: changed_capabilities(&installed_caps, &other_caps),
        }
    }

    /// The other version may do something the installed one may not that the user is warned
    /// about: a risky host function, or a host entry that admits almost any site.
    pub fn adds_risk(&self) -> bool {
        !self.added_risks.is_empty() || self.added_allowed_hosts.iter().any(|host| host.wide)
    }

    /// Both versions ask for the same: host functions, hosts and capabilities.
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

fn host_functions(manifest: &PluginManifest) -> BTreeSet<&str> {
    manifest
        .api
        .host_functions
        .iter()
        .flatten()
        .map(String::as_str)
        .collect()
}

fn allowed_hosts(manifest: &PluginManifest) -> BTreeSet<&str> {
    manifest
        .runtime
        .allowed_hosts
        .iter()
        .map(String::as_str)
        .collect()
}

/// What `from` has that `to` lacks.
fn function_changes(from: &BTreeSet<&str>, to: &BTreeSet<&str>) -> Vec<HostFunctionChange> {
    from.difference(to)
        .map(|name| HostFunctionChange {
            name: (*name).to_owned(),
            risk: match classify_host_function(name) {
                Some(HostFunctionClass::Risky(risk)) => Some(risk),
                Some(HostFunctionClass::Plain) | None => None,
            },
        })
        .collect()
}

/// One capability as a whole, so that two of the same kind and id can be told apart.
#[derive(PartialEq)]
enum CapabilityRef<'a> {
    Importer(&'a PluginImporterCapability),
    Updater(&'a PluginUpdaterCapability),
    ContentProvider(&'a PluginContentProviderCapability),
}

struct CapabilityEntry<'a> {
    kind: CapabilityKind,
    id: &'a str,
    name: &'a str,
    whole: CapabilityRef<'a>,
}

impl CapabilityEntry<'_> {
    fn is_same(&self, other: &Self) -> bool {
        self.kind == other.kind && self.id == other.id
    }

    fn change(&self) -> CapabilityChange {
        CapabilityChange {
            kind: self.kind,
            id: self.id.to_owned(),
            name: self.name.to_owned(),
        }
    }
}

fn capability_entries<'a, T: AsCapabilityMetadata>(
    kind: CapabilityKind,
    list: &'a [T],
    whole: fn(&'a T) -> CapabilityRef<'a>,
) -> impl Iterator<Item = CapabilityEntry<'a>> {
    list.iter().map(move |c| {
        let metadata = c.as_metadata();
        CapabilityEntry {
            kind,
            id: metadata.id.as_str(),
            name: metadata.name.as_str(),
            whole: whole(c),
        }
    })
}

/// Sorted by kind and id, the first of each kept: the one the launcher registers.
fn capabilities(capabilities: Option<&PluginCapabilities>) -> Vec<CapabilityEntry<'_>> {
    let Some(capabilities) = capabilities else {
        return Vec::new();
    };

    let mut all: Vec<_> = capability_entries(
        CapabilityKind::Importer,
        &capabilities.importers,
        CapabilityRef::Importer,
    )
    .chain(capability_entries(
        CapabilityKind::Updater,
        &capabilities.updaters,
        CapabilityRef::Updater,
    ))
    .chain(capability_entries(
        CapabilityKind::ContentProvider,
        &capabilities.content_providers,
        CapabilityRef::ContentProvider,
    ))
    .collect();
    all.sort_by(|a, b| (a.kind, a.id).cmp(&(b.kind, b.id)));
    all.dedup_by(|a, b| a.is_same(b));
    all
}

/// What `from` has that `to` lacks.
fn capability_changes(from: &[CapabilityEntry], to: &[CapabilityEntry]) -> Vec<CapabilityChange> {
    from.iter()
        .filter(|c| !to.iter().any(|o| c.is_same(o)))
        .map(CapabilityEntry::change)
        .collect()
}

/// What both have under the same kind and id but declare differently: a content provider that
/// now serves another platform, other file extensions, other handlers. Named as `other` has it.
fn changed_capabilities(
    installed: &[CapabilityEntry],
    other: &[CapabilityEntry],
) -> Vec<CapabilityChange> {
    other
        .iter()
        .filter(|c| installed.iter().any(|i| c.is_same(i) && c.whole != i.whole))
        .map(CapabilityEntry::change)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::instance::{
        CapabilityId, CapabilityMetadata, ContentProviderCapabilityMetadata,
        ImporterCapabilityMetadata, PlatformId, UpdaterCapabilityMetadata,
    };
    use crate::features::plugins::ProviderHandlers;

    fn manifest(hosts: &[&str], functions: Option<&[&str]>) -> PluginManifest {
        let json = serde_json::json!({
            "metadata": {
                "id": "test-plugin",
                "name": "Test Plugin",
                "version": "1.0.0",
                "description": null,
                "authors": [],
                "license": null
            },
            "runtime": { "allowedHosts": hosts, "allowedPaths": [] },
            "load": { "type": "extism", "file": "plugin.wasm", "memoryLimit": null },
            "api": { "version": "*", "hostFunctions": functions },
        });
        let dto: aether_core_plugin_api::v0::PluginManifestDto =
            serde_json::from_value(json).unwrap();
        dto.try_into().unwrap()
    }

    fn base(id: &str, name: &str) -> CapabilityMetadata {
        CapabilityMetadata {
            id: id.parse::<CapabilityId>().unwrap(),
            name: name.to_owned(),
            description: None,
            icon: None,
        }
    }

    fn importer(id: &str) -> PluginImporterCapability {
        PluginImporterCapability {
            metadata: ImporterCapabilityMetadata {
                base: base(id, &format!("{id} importer")),
                field_label: None,
                supported_extensions: vec![],
            },
            handler: "import".to_owned(),
        }
    }

    fn updater(id: &str) -> PluginUpdaterCapability {
        PluginUpdaterCapability {
            metadata: UpdaterCapabilityMetadata {
                base: base(id, &format!("{id} updater")),
            },
            handler: "update".to_owned(),
        }
    }

    #[test]
    fn should_report_nothing_when_both_versions_ask_for_the_same() {
        let installed = manifest(&["api.example.com"], Some(&["log", "run_command"]));
        let other = manifest(&["api.example.com"], Some(&["run_command", "log"]));

        let diff = PluginUpdateDiff::between(&installed, None, &other, None);

        assert!(diff.is_empty());
        assert!(!diff.adds_risk());
    }

    #[test]
    fn should_report_the_risk_of_each_host_function_when_the_new_version_adds_it() {
        let installed = manifest(&[], Some(&["log"]));
        let other = manifest(&[], Some(&["log", "write_instance_file", "http_get"]));

        let diff = PluginUpdateDiff::between(&installed, None, &other, None);

        assert_eq!(
            diff.added_host_functions,
            [
                HostFunctionChange {
                    name: "http_get".into(),
                    risk: None
                },
                HostFunctionChange {
                    name: "write_instance_file".into(),
                    risk: Some(HostFunctionRisk::WritesInstanceFiles)
                },
            ]
        );
        assert_eq!(diff.added_risks, [HostFunctionRisk::WritesInstanceFiles]);
        assert!(diff.removed_host_functions.is_empty());
        assert!(diff.adds_risk());
    }

    #[test]
    fn should_add_no_risk_when_the_installed_version_already_carries_it() {
        let installed = manifest(&[], Some(&["run_command"]));
        let other = manifest(&[], Some(&["run_command", "install_java"]));

        let diff = PluginUpdateDiff::between(&installed, None, &other, None);

        assert_eq!(diff.added_risks, [HostFunctionRisk::InstallsJava]);

        let only_same_risk = PluginUpdateDiff::between(&other, None, &other, None);
        assert!(only_same_risk.added_risks.is_empty());
    }

    #[test]
    fn should_report_removed_host_functions_without_a_new_risk_when_the_new_version_drops_them() {
        let installed = manifest(&[], Some(&["log", "run_command"]));
        let other = manifest(&[], Some(&["log"]));

        let diff = PluginUpdateDiff::between(&installed, None, &other, None);

        assert_eq!(
            diff.removed_host_functions,
            [HostFunctionChange {
                name: "run_command".into(),
                risk: Some(HostFunctionRisk::RunsPrograms)
            }]
        );
        assert!(diff.added_host_functions.is_empty());
        assert!(!diff.adds_risk());
    }

    #[test]
    fn should_compare_an_undeclared_list_of_host_functions_as_an_empty_one() {
        let installed = manifest(&[], None);
        let other = manifest(&[], Some(&["run_command"]));

        let diff = PluginUpdateDiff::between(&installed, None, &other, None);

        assert_eq!(diff.added_risks, [HostFunctionRisk::RunsPrograms]);
    }

    #[test]
    fn should_flag_a_wide_host_when_the_new_version_adds_it() {
        let installed = manifest(&["api.example.com"], Some(&[]));
        let other = manifest(&["api.example.com", "*", "cdn.example.com"], Some(&[]));

        let diff = PluginUpdateDiff::between(&installed, None, &other, None);

        assert_eq!(
            diff.added_allowed_hosts,
            [
                AllowedHostChange {
                    entry: "*".into(),
                    wide: true
                },
                AllowedHostChange {
                    entry: "cdn.example.com".into(),
                    wide: false
                },
            ]
        );
        assert!(diff.removed_allowed_hosts.is_empty());
        assert!(diff.adds_risk());
    }

    #[test]
    fn should_not_flag_a_risk_when_only_narrow_hosts_change() {
        let installed = manifest(&["old.example.com"], Some(&[]));
        let other = manifest(&["new.example.com"], Some(&[]));

        let diff = PluginUpdateDiff::between(&installed, None, &other, None);

        assert_eq!(diff.removed_allowed_hosts, ["old.example.com"]);
        assert_eq!(diff.added_allowed_hosts.len(), 1);
        assert!(!diff.adds_risk());
        assert!(!diff.is_empty());
    }

    #[test]
    fn should_report_capabilities_by_kind_and_id_when_they_change() {
        let plain = manifest(&[], Some(&[]));
        let installed = PluginCapabilities {
            importers: vec![importer("packwiz")],
            updaters: vec![updater("packwiz")],
            ..Default::default()
        };
        let other = PluginCapabilities {
            importers: vec![importer("packwiz"), importer("mrpack")],
            ..Default::default()
        };

        let diff = PluginUpdateDiff::between(&plain, Some(&installed), &plain, Some(&other));

        assert_eq!(
            diff.added_capabilities,
            [CapabilityChange {
                kind: CapabilityKind::Importer,
                id: "mrpack".into(),
                name: "mrpack importer".into()
            }]
        );
        assert_eq!(
            diff.removed_capabilities,
            [CapabilityChange {
                kind: CapabilityKind::Updater,
                id: "packwiz".into(),
                name: "packwiz updater".into()
            }]
        );
        assert!(!diff.adds_risk());
    }

    #[test]
    fn should_report_every_capability_as_added_when_the_installed_version_has_none() {
        let plain = manifest(&[], Some(&[]));
        let other = PluginCapabilities {
            updaters: vec![updater("packwiz")],
            ..Default::default()
        };

        let diff = PluginUpdateDiff::between(&plain, None, &plain, Some(&other));

        assert_eq!(diff.added_capabilities.len(), 1);
        assert!(diff.removed_capabilities.is_empty());
    }

    fn content_provider(platform: Option<&str>) -> PluginContentProviderCapability {
        PluginContentProviderCapability {
            metadata: ContentProviderCapabilityMetadata {
                base: base("mods", "Mods"),
                platform: platform.map(|p| p.parse::<PlatformId>().unwrap()),
            },
            handlers: ProviderHandlers {
                search: "search".to_owned(),
                get_content: "get_content".to_owned(),
                list_version: None,
                check_compatibility: None,
                resolve_file: Some("resolve_file".to_owned()),
                resolve_install_plan: None,
                resolve_pack_metadata: None,
                get_contents_metadata: None,
                get_versions_metadata: None,
                get_authors: None,
                get_version: None,
            },
        }
    }

    /// Same id, another platform: the provider is then asked for every reference to that
    /// platform, so it is not the same capability any more.
    #[test]
    fn should_report_a_changed_capability_when_a_provider_keeps_its_id_and_changes_platform() {
        let plain = manifest(&[], Some(&[]));
        let installed = PluginCapabilities {
            content_providers: vec![content_provider(Some("curseforge"))],
            ..Default::default()
        };
        let other = PluginCapabilities {
            content_providers: vec![content_provider(Some("modrinth"))],
            ..Default::default()
        };

        let diff = PluginUpdateDiff::between(&plain, Some(&installed), &plain, Some(&other));

        assert_eq!(
            diff.changed_capabilities,
            [CapabilityChange {
                kind: CapabilityKind::ContentProvider,
                id: "mods".into(),
                name: "Mods".into()
            }]
        );
        assert!(diff.added_capabilities.is_empty());
        assert!(diff.removed_capabilities.is_empty());
        assert!(!diff.is_empty());
    }

    #[test]
    fn should_report_no_changed_capability_when_it_is_declared_the_same() {
        let plain = manifest(&[], Some(&[]));
        let capabilities = PluginCapabilities {
            content_providers: vec![content_provider(Some("curseforge"))],
            ..Default::default()
        };

        let diff =
            PluginUpdateDiff::between(&plain, Some(&capabilities), &plain, Some(&capabilities));

        assert!(diff.is_empty());
    }
}
