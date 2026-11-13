use super::PluginSourceType;

/// Why the loader could not bring a plugin up. The variant is what the user is told; the
/// arguments are what it was told about.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum PluginLoadFailure {
    #[error(
        "wasm integrity check failed: '{path}' hashes to {actual_sha256} but {recorded_sha256} was \
         recorded when the plugin was installed from {source_type}. The plugin binary changed on \
         disk — reinstall the plugin from its source before enabling it."
    )]
    HashMismatch {
        path: String,
        actual_sha256: String,
        recorded_sha256: String,
        source_type: PluginSourceType,
    },

    /// The runtime refused the wasm. Its text is the runtime's own, not ours to categorize.
    #[error("{0}")]
    Build(String),

    #[error("the application container is not available")]
    ContainerUnavailable,

    /// Any other reason the plugin did not load (settings, storage, loader lookup, ...). Its text
    /// is the source error's own, not ours to categorize.
    #[error("{0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_name_the_digests_and_the_source_when_a_hash_mismatch_is_written_out() {
        let failure = PluginLoadFailure::HashMismatch {
            path: "p/a.wasm".to_owned(),
            actual_sha256: "aaaa".to_owned(),
            recorded_sha256: "bbbb".to_owned(),
            source_type: PluginSourceType::GitHub,
        };

        let text = failure.to_string();

        assert!(
            ["p/a.wasm", "aaaa", "bbbb", "github"]
                .iter()
                .all(|part| text.contains(part)),
            "{text}"
        );
    }
}
