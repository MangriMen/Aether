use std::path::{Path, PathBuf};

use async_trait::async_trait;
use futures::StreamExt;

use crate::{
    features::java::{
        Java, JavaDomainError, JavaInstallationService,
        domain::extract_java_major_minor_version,
        infra::{JavaProperties, get_java_properties},
    },
    shared,
};

use super::{JAVA_BIN, JAVA_WINDOW_BIN};

pub struct FsJavaInstallationService;

impl FsJavaInstallationService {
    /// Ensures that the given path ends with `JAVA_WINDOW_BIN`.
    fn get_java_window_bin_path(path: PathBuf) -> Result<PathBuf, JavaDomainError> {
        match path.file_name() {
            Some(file_name) if file_name == JAVA_WINDOW_BIN => Ok(path),
            Some(_) => Ok(path.join(JAVA_WINDOW_BIN)),
            None => Err(JavaDomainError::InvalidPath { path }),
        }
    }

    async fn scan_single_dir(&self, base_path: &Path) -> Result<Vec<Java>, JavaDomainError> {
        let mut found_java = Vec::new();

        let mut entries =
            tokio::fs::read_dir(base_path)
                .await
                .map_err(|_| JavaDomainError::InvalidPath {
                    path: base_path.to_path_buf(),
                })?;

        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|_| JavaDomainError::InvalidPath {
                    path: base_path.to_path_buf(),
                })?
        {
            let path = entry.path();
            if let Ok(metadata) = tokio::fs::metadata(&path).await
                && metadata.is_dir()
            {
                let potential_bin_dir = path.join("bin");

                let search_path = if tokio::fs::try_exists(&potential_bin_dir)
                    .await
                    .unwrap_or(false)
                {
                    potential_bin_dir
                } else {
                    path
                };

                if let Ok(java) = self.locate_java(&search_path).await {
                    found_java.push(java);
                }
            }
        }
        Ok(found_java)
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

        if !tokio::fs::try_exists(&java_window_bin_path)
            .await
            .unwrap_or(false)
        {
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
                    version.clone(),
                    architecture.clone(),
                    java_window_bin_path.to_string_lossy().to_string(),
                )
            })
        } else {
            Err(JavaDomainError::InvalidPath {
                path: path.to_path_buf(),
            })
        }
    }

    async fn discover_installations(
        &self,
        base_paths: &[PathBuf],
    ) -> Result<Vec<Java>, JavaDomainError> {
        // We convert the slice into owned PathBufs immediately to cut off lifetime constraints.
        let scan_stream = futures::stream::iter(base_paths.to_vec())
            .map(|path| async move {
                // Check metadata concurrently inside the pipeline
                if let Ok(meta) = tokio::fs::metadata(&path).await
                    && meta.is_dir()
                {
                    return self.scan_single_dir(&path).await;
                }
                Ok(Vec::new()) // Safely return an empty vec if path is invalid/inaccessible
            })
            .buffer_unordered(4);

        let mut all_found = scan_stream
            .fold(
                Vec::with_capacity(base_paths.len() * 2),
                |mut acc, scan_result| async move {
                    if let Ok(mut java_list) = scan_result {
                        acc.append(&mut java_list);
                    }
                    acc
                },
            )
            .await;

        all_found.sort_by(|a, b| a.path().cmp(b.path()));
        all_found.dedup_by(|a, b| a.path() == b.path());

        Ok(all_found)
    }
}
