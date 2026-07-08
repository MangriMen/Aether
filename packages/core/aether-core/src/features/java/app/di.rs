use std::sync::Arc;

use crate::features::java::app::ports::{
    DiscoverJavaUseCasePort, EditJavaUseCasePort, GetActiveJavaInstallationsUseCasePort,
    JavaInstallService, JavaInstallationService, JavaInstallationTracker, JavaQueryService,
    JavaStorage, JreProvider, ListJavaUseCasePort, RemoveJavaUseCasePort, TestJreUseCasePort,
};

/// Extension trait providing access to all Java feature use cases and services.
///
/// Implemented on the core dependency injection container to expose
/// Java-specific functionality in a centralized manner.
pub trait JavaFeature {
    // ── Use cases ──
    fn discover_java_use_case(&self) -> Arc<dyn DiscoverJavaUseCasePort>;
    fn edit_java_use_case(&self) -> Arc<dyn EditJavaUseCasePort>;
    fn get_active_java_installations_use_case(
        &self,
    ) -> Arc<dyn GetActiveJavaInstallationsUseCasePort>;
    fn list_java_use_case(&self) -> Arc<dyn ListJavaUseCasePort>;
    fn remove_java_use_case(&self) -> Arc<dyn RemoveJavaUseCasePort>;
    fn test_jre_use_case(&self) -> Arc<dyn TestJreUseCasePort>;

    // ── Ports (use cases that also implement service traits) ──
    fn get_java_use_case(&self) -> Arc<dyn JavaQueryService>;
    fn install_java_use_case(&self) -> Arc<dyn JavaInstallService>;

    // ── Ports / services ──
    fn java_installation_service(&self) -> Arc<dyn JavaInstallationService>;
    fn java_installation_tracker(&self) -> Arc<dyn JavaInstallationTracker>;
    fn java_storage(&self) -> Arc<dyn JavaStorage>;
    fn jre_provider(&self) -> Arc<dyn JreProvider>;
}
