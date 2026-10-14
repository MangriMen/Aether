mod java;
mod java_installation_guard;

pub use java::Java;
pub(crate) use java::{CUSTOM_JAVA_VERSION, UNKNOWN_JAVA_ARCHITECTURE};
pub use java_installation_guard::JavaInstallationGuard;
