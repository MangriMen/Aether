/// The current version of the plugin API, sourced from `Cargo.toml`.
/// This is the single source of truth for the API version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod v0;
