use std::sync::Arc;

use async_trait::async_trait;

use crate::features::java::{
    Java, JavaInstallationService, JavaStorage,
    app::{JavaApplicationError, dtos::EditJava, ports::EditJavaUseCasePort},
};

pub struct EditJavaUseCase {
    java_storage: Arc<dyn JavaStorage>,
    java_installation_service: Arc<dyn JavaInstallationService>,
}

impl EditJavaUseCase {
    pub fn new(
        java_storage: Arc<dyn JavaStorage>,
        java_installation_service: Arc<dyn JavaInstallationService>,
    ) -> Self {
        Self {
            java_storage,
            java_installation_service,
        }
    }
}

#[async_trait]
impl EditJavaUseCasePort for EditJavaUseCase {
    async fn execute(&self, edit_java: EditJava) -> Result<Java, JavaApplicationError> {
        // What the core located, never what the user typed: a directory typed in would be stored
        // as is and fail at launch, and a path the core does not recognise as a Java must not be
        // stored at all, since plugins may start programs from the registered Java paths.
        let located_java = self
            .java_installation_service
            .locate_java(edit_java.path.as_path())
            .await?;

        let java_to_upsert = if located_java.major_version() == edit_java.major_version {
            located_java
        } else {
            Java::new(
                edit_java.major_version,
                located_java.version().to_owned(),
                located_java.architecture().to_owned(),
                located_java.path().to_owned(),
            )
        };

        let saved_java = self.java_storage.upsert(java_to_upsert).await?;

        Ok(saved_java)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        path::{Path, PathBuf},
        sync::Mutex,
    };

    use super::*;
    use crate::features::java::{JavaDomainError, JavaPath};

    #[derive(Default)]
    struct MemStorage(Mutex<HashMap<u32, Java>>);

    #[async_trait]
    impl JavaStorage for MemStorage {
        async fn list(&self) -> Result<Vec<Java>, JavaDomainError> {
            Ok(self.0.lock().unwrap().values().cloned().collect())
        }

        async fn get(&self, version: u32) -> Result<Option<Java>, JavaDomainError> {
            Ok(self.0.lock().unwrap().get(&version).cloned())
        }

        async fn upsert(&self, java: Java) -> Result<Java, JavaDomainError> {
            self.0
                .lock()
                .unwrap()
                .insert(java.major_version(), java.clone());
            Ok(java)
        }

        async fn remove(&self, version: u32) -> Result<(), JavaDomainError> {
            self.0.lock().unwrap().remove(&version);
            Ok(())
        }
    }

    /// Recognises only `/jdk/bin/java` as a Java 21, whatever the path it is asked about.
    struct OneJava;

    #[async_trait]
    impl JavaInstallationService for OneJava {
        async fn locate_java(&self, path: &Path) -> Result<Java, JavaDomainError> {
            if path.starts_with(recognised_dir()) {
                Ok(Java::new(
                    21,
                    "21.0.1".into(),
                    "x64".into(),
                    JavaPath::try_from(recognised_java().as_path())?,
                ))
            } else {
                Err(JavaDomainError::InvalidPath {
                    path: path.to_path_buf(),
                })
            }
        }

        async fn discover_installations(
            &self,
            _: &[PathBuf],
        ) -> Result<Vec<Java>, JavaDomainError> {
            Ok(Vec::new())
        }
    }

    fn recognised_dir() -> PathBuf {
        std::env::temp_dir()
            .join("aether-edit-java-test")
            .join("bin")
    }

    fn recognised_java() -> PathBuf {
        recognised_dir().join("java")
    }

    fn use_case() -> (EditJavaUseCase, Arc<MemStorage>) {
        let storage = Arc::new(MemStorage::default());
        (
            EditJavaUseCase::new(storage.clone(), Arc::new(OneJava)),
            storage,
        )
    }

    fn edit(major_version: u32, path: &Path) -> EditJava {
        EditJava {
            major_version,
            path: JavaPath::try_from(path).unwrap(),
        }
    }

    #[tokio::test]
    async fn should_not_store_a_path_when_it_is_not_recognised_as_a_java() {
        let (use_case, storage) = use_case();
        let not_java = std::env::temp_dir().join("elsewhere").join("bash");

        let result = use_case.execute(edit(21, &not_java)).await;

        assert!(matches!(
            result,
            Err(JavaApplicationError::Domain(
                JavaDomainError::InvalidPath { .. }
            ))
        ));
        assert!(storage.list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn should_store_the_located_path_when_a_directory_is_given() {
        let (use_case, storage) = use_case();

        let saved = use_case
            .execute(edit(21, &recognised_dir()))
            .await
            .expect("recognised");

        assert_eq!(saved.path().as_path(), recognised_java().as_path());
        assert_eq!(storage.list().await.unwrap(), vec![saved]);
    }

    #[tokio::test]
    async fn should_store_the_located_path_when_the_major_version_is_overridden() {
        let (use_case, _) = use_case();

        let saved = use_case
            .execute(edit(17, &recognised_dir()))
            .await
            .expect("recognised");

        assert_eq!(saved.major_version(), 17);
        assert_eq!(saved.path().as_path(), recognised_java().as_path());
    }
}
