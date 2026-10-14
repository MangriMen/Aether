use crate::features::instance::ProviderId;

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("Unsupported API version")]
    UnsupportedApi,

    #[error("Invalid semver in {field}: \"{value}\" — {error}")]
    InvalidSemver {
        field: String,
        value: String,
        error: String,
    },

    #[error("Invalid path mapping")]
    InvalidPathMapping,

    #[error(
        "Plugin declares host functions this launcher does not provide: {}. \
         It was probably built for a newer version of the launcher",
        quote_names(.0)
    )]
    UnknownHostFunctions(Vec<String>),

    #[error(
        "Plugin manifest does not declare `api.hostFunctions`, which this launcher requires. \
         The plugin was built against an older plugin API; rebuild it against the current \
         `aether-core-plugin-api`"
    )]
    HostFunctionsNotDeclared,

    #[error(
        "Invalid {field} \"{value}\": an id may contain only lowercase Latin letters, digits, \
         `_` and `-`, and must start with a letter or digit"
    )]
    InvalidId { field: String, value: String },

    #[error(
        "Plugin id \"{}\" is reserved for providers built into Aether",
        ProviderId::BUILTIN_PLUGIN_ID
    )]
    ReservedPluginId,

    #[error(
        "Content provider \"{capability_id}\" declares a `platform` but no `resolveFile` handler, \
         which a platform requires: references to the platform's content are resolved through it"
    )]
    PlatformWithoutResolveFile { capability_id: String },
}

fn quote_names(names: &[String]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}
