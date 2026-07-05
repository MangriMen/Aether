mod app;
mod domain;
pub mod infra;

// Expose only what the outside world needs — explicit facade (Rules 4.1–4.4)
pub use app::{
    // Use Cases (Inbound Ports)
    DiscoverJavaUseCase,
    // DTOs (re-exported by app/mod.rs via pub use dtos::*)
    EditJava,
    EditJavaUseCase,
    GetActiveJavaInstallationsUseCase,
    GetJavaUseCase,
    InstallJava,
    InstallJavaUseCase,
    // Application Error
    JavaApplicationError,
    // Outbound Ports (re-exported explicitly for cross-feature consumption)
    JavaInstallService,
    JavaInstallationService,
    JavaInstallationTracker,
    JavaQueryService,
    JavaStorage,
    JreProvider,
    ListJavaUseCase,
    RemoveJavaUseCase,
    TestJreUseCase,
};
pub use domain::{
    // Anemic Domain Models (Rule 4.4)
    Java,
    // Domain Error
    JavaDomainError,
    JavaInstallationGuard,
};
