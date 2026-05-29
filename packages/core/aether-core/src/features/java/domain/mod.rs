#[cfg(test)]
mod __tests__;
mod error;
mod extract_java_major_minor_version;
mod models;

pub use error::JavaDomainError;
pub use extract_java_major_minor_version::extract_java_major_minor_version;
pub use models::{CUSTOM_JAVA_VERSION, Java, JavaInstallationGuard, UNKNOWN_JAVA_ARCHITECTURE};
