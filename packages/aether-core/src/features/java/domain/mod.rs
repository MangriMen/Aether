#[cfg(test)]
mod __tests__;
mod error;
mod extract_java_major_minor_version;
mod models;

pub use error::JavaDomainError;
pub(crate) use extract_java_major_minor_version::extract_java_major_minor_version;
pub(crate) use models::{CUSTOM_JAVA_VERSION, UNKNOWN_JAVA_ARCHITECTURE};
pub use models::{Java, JavaInstallationGuard};
