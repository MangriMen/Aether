//! The keys of the Extism config the host gives every plugin when it loads it. A plugin reads
//! them with `extism_pdk::config::get`, or with `plugin_id()` and `host_api_version()` of
//! `aether-core-plugin-pdk`.

/// The id of the plugin, as the host knows it.
pub const CONFIG_PLUGIN_ID: &str = "aether.pluginId";

/// The version of this API the host implements (`MAJOR.MINOR.PATCH`), the one `api.version` of
/// the manifest is checked against.
pub const CONFIG_API_VERSION: &str = "aether.apiVersion";
