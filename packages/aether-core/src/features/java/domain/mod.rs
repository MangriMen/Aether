mod error;
mod models;

pub use error::JavaDomainError;
pub use models::{
    InvalidJavaPath, Java, JavaInstallationGuard, JavaPath, JavaPathRejection, JavaVersion,
};
