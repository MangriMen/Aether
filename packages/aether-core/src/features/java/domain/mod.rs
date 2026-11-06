mod error;
mod models;

pub use error::JavaDomainError;
pub(crate) use models::{CUSTOM_JAVA_VERSION, UNKNOWN_JAVA_ARCHITECTURE};
pub use models::{
    InvalidJavaPath, Java, JavaInstallationGuard, JavaPath, JavaPathRejection, JavaVersion,
};
