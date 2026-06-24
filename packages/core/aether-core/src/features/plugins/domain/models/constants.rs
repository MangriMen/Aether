use std::sync::LazyLock;

/// The current API version, sourced from `aether-core-plugin-api`'s `Cargo.toml`.
/// This is the single source of truth — update the version in
/// `packages/core/aether-core-plugin-api/Cargo.toml` to change it.
pub static PLUGIN_API_VERSION: LazyLock<semver::Version> = LazyLock::new(|| {
    semver::Version::parse(aether_core_plugin_api::VERSION)
        .expect("Invalid plugin API version in aether-core-plugin-api Cargo.toml")
});
