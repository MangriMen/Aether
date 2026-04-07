use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{
    features::java::{
        infra::{get_java_properties, JavaProperties},
        utils::extract_java_major_minor_version,
        Java, JavaDomainError, JavaInstallationService,
    },
    shared,
};

use super::{JAVA_BIN, JAVA_WINDOW_BIN};

pub struct FsJavaInstallationService;

impl FsJavaInstallationService {
    /// Ensures that the given path ends with `JAVA_WINDOW_BIN`.
    ///
    /// If the provided path does not already end with `JAVA_WINDOW_BIN`,
    /// this function appends it to the path. Otherwise, it returns the path unchanged.
    fn get_java_window_bin_path(path: PathBuf) -> Result<PathBuf, JavaDomainError> {
        match path.file_name() {
            Some(file_name) => {
                if file_name.to_string_lossy() != JAVA_WINDOW_BIN {
                    Ok(path.join(JAVA_WINDOW_BIN))
                } else {
                    Ok(path)
                }
            }
            None => Err(JavaDomainError::InvalidPath { path }),
        }
    }
}

#[async_trait]
impl JavaInstallationService for FsJavaInstallationService {
    /// Attempts to resolve the given file path and retrieve the Java version located at this path.
    ///
    /// Returns `None` if the path does not exist or if a valid Java installation is not found at the specified path.
    async fn locate_java(&self, path: &Path) -> Result<Java, JavaDomainError> {
        // Attempt to canonicalize the potential Java filepath
        // If it fails, return None (Java is not here)
        let canonical_path =
            shared::canonicalize(path).map_err(|_| JavaDomainError::InvalidPath {
                path: path.to_path_buf(),
            })?;

        let java_window_bin_path = Self::get_java_window_bin_path(canonical_path)?;
        if !java_window_bin_path.exists() {
            return Err(JavaDomainError::InvalidPath {
                path: path.to_path_buf(),
            });
        }

        // Create the path for the Java binary (replacing JAVA_WINDOW_BIN with JAVA_BIN)
        let java_bin_path = java_window_bin_path.with_file_name(JAVA_BIN);

        // Get the Java version and architecture
        let JavaProperties {
            version,
            architecture,
        } = get_java_properties(&java_bin_path)?;

        // Extract version and architecture information
        if let (Some(version), Some(architecture)) = (version, architecture) {
            extract_java_major_minor_version(&version).map(|(_, major_version)| {
                Java::new(
                    major_version,
                    version.to_string(),
                    architecture.to_string(),
                    java_window_bin_path.to_string_lossy().to_string(),
                )
            })
        } else {
            Err(JavaDomainError::InvalidPath {
                path: path.to_path_buf(),
            })
        }
    }
}
