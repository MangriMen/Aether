use std::sync::Arc;

use crate::features::java::{
    CUSTOM_JAVA_VERSION, Java, JavaDomainError, JavaInstallationService, JavaStorage,
    UNKNOWN_JAVA_ARCHITECTURE,
    app::{JavaApplicationError, dtos::EditJava},
};

pub struct EditJavaUseCase<JS: JavaStorage, JIS: JavaInstallationService> {
    java_storage: Arc<JS>,
    java_installation_service: Arc<JIS>,
}

impl<JS: JavaStorage, JIS: JavaInstallationService> EditJavaUseCase<JS, JIS> {
    pub fn new(java_storage: Arc<JS>, java_installation_service: Arc<JIS>) -> Self {
        Self {
            java_storage,
            java_installation_service,
        }
    }

    pub async fn execute(&self, edit_java: EditJava) -> Result<Java, JavaApplicationError> {
        if edit_java.path.to_string_lossy().is_empty() {
            return Err(JavaDomainError::EmptyPath)?;
        }

        let java_to_upsert = match self
            .java_installation_service
            .locate_java(&edit_java.path)
            .await
        {
            Ok(located_java) if located_java.major_version() == edit_java.major_version => {
                located_java
            }
            Ok(located_java) => Java::new(
                edit_java.major_version,
                located_java.version().to_owned(),
                located_java.architecture().to_owned(),
                edit_java.path.to_string_lossy().to_string(),
            ),
            Err(_) => Java::new(
                edit_java.major_version,
                CUSTOM_JAVA_VERSION.to_owned(),
                UNKNOWN_JAVA_ARCHITECTURE.to_owned(),
                edit_java.path.to_string_lossy().to_string(),
            ),
        };

        let saved_java = self.java_storage.upsert(java_to_upsert).await?;

        Ok(saved_java)
    }
}
