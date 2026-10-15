use std::collections::HashSet;
use std::path::PathBuf;

use async_trait::async_trait;

use crate::features::java::app::EditJava;
use crate::features::java::{Java, JavaApplicationError};

/// Port trait for discovering Java installations on the system.
#[async_trait]
pub trait DiscoverJavaUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<Vec<Java>, JavaApplicationError>;
}

/// Port trait for editing (adding or updating) a Java runtime.
#[async_trait]
pub trait EditJavaUseCasePort: Send + Sync {
    async fn execute(&self, edit_java: EditJava) -> Result<Java, JavaApplicationError>;
}

/// Port trait for querying currently locked / active Java versions.
#[async_trait]
pub trait GetActiveJavaInstallationsUseCasePort: Send + Sync {
    async fn execute(&self) -> HashSet<u32>;
}

/// Port trait for listing all stored Java runtimes.
#[async_trait]
pub trait ListJavaUseCasePort: Send + Sync {
    async fn execute(&self) -> Result<Vec<Java>, JavaApplicationError>;
}

/// Port trait for removing a stored Java runtime by version.
#[async_trait]
pub trait RemoveJavaUseCasePort: Send + Sync {
    async fn execute(&self, major_version: u32) -> Result<(), JavaApplicationError>;
}

/// Port trait for testing a JRE at a given path.
#[async_trait]
pub trait TestJreUseCasePort: Send + Sync {
    async fn execute(&self, path: PathBuf) -> Result<Java, JavaApplicationError>;
}
