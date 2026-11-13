use crate::features::plugins::{PluginCapabilities, PluginHandle};

use super::{ManifestError, PluginLoadFailure, PluginManifest};

#[derive(Clone)]
pub enum PluginState {
    NotLoaded,
    Loading,
    Loaded(PluginHandle),
    Unloading,
    /// Its files are being deleted. Nothing loads, unloads or syncs the plugin meanwhile; it
    /// leaves this state by being forgotten, or by being put back as it was if the delete fails.
    Removing,
    /// The plugin asks for what this host does not offer: an API version or host functions.
    Incompatible(PluginStateReason),
    Failed(PluginStateReason),
}

impl PluginState {
    /// The explanation carried by `Incompatible` and `Failed`.
    pub fn reason(&self) -> Option<&PluginStateReason> {
        match self {
            Self::Incompatible(reason) | Self::Failed(reason) => Some(reason),
            _ => None,
        }
    }
}

/// Why the core refused to run a plugin. The variant is what the user is told in their own
/// language. Held in memory only: it is neither stored nor sent to plugins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStateReason {
    IncompatibleApiVersion {
        required: semver::VersionReq,
        host: semver::Version,
    },
    InvalidManifest {
        error: ManifestError,
    },
    LoadFailed {
        reason: PluginLoadFailure,
    },
}

impl std::fmt::Display for PluginStateReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncompatibleApiVersion { required, host } => {
                write!(
                    f,
                    "Plugin requires API {required}, host API version is {host}"
                )
            }
            Self::InvalidManifest { error } => {
                write!(f, "Plugin manifest failed validation: {error}")
            }
            Self::LoadFailed { reason } => write!(f, "Plugin failed to load: {reason}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_name_both_versions_when_an_api_version_reason_is_written_out() {
        let reason = PluginStateReason::IncompatibleApiVersion {
            required: semver::VersionReq::parse("^2.0.0").unwrap(),
            host: semver::Version::new(1, 4, 0),
        };

        let text = reason.to_string();

        assert!(text.contains("^2.0.0") && text.contains("1.4.0"), "{text}");
    }

    #[test]
    fn should_keep_the_details_when_a_load_failure_is_written_out() {
        let reason = PluginStateReason::LoadFailed {
            reason: PluginLoadFailure::Build("the wasm does not compile".to_owned()),
        };

        assert!(
            reason.to_string().contains("the wasm does not compile"),
            "{reason}"
        );
    }
}

#[derive(Clone)]
pub struct Plugin {
    pub manifest: PluginManifest,
    pub capabilities: Option<PluginCapabilities>,
    pub hash: String,
    /// Whether a trust-on-first-use record exists for this plugin, i.e. it was
    /// installed through a remote provider and its `.wasm` is checked on load.
    pub verified: bool,
    pub state: PluginState,
}
