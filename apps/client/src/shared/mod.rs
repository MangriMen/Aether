pub mod commands;
mod idempotency_manager;
pub mod reveal_in_explorer;
pub mod specta;
pub mod tauri_paths;

// Re-export from idempotency_manager: explicit to avoid ambiguous glob conflicts
pub use idempotency_manager::infra::TauriIdempotencyExt;
pub use idempotency_manager::{
    ActiveRequest, IdempotencyManager, IdempotencyManagerError, RequestId,
};
pub use reveal_in_explorer::infra::reveal_in_explorer;
