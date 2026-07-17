use std::collections::HashMap;
use std::sync::Arc;

use crate::features::instance::app::ports::PackLifecycleHandler;
use crate::features::instance::domain::InstanceError;

/// Registry of `PackLifecycleHandler` implementations, keyed by `format_id`.
///
/// Used by the `InstallContentUseCase` to dispatch modpack manifests to the
/// correct handler after receiving them from a `ContentSource`.
#[derive(Default)]
pub struct PackLifecycleHandlerRegistry {
    handlers: HashMap<String, Arc<dyn PackLifecycleHandler>>,
}

impl PackLifecycleHandlerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, handler: Arc<dyn PackLifecycleHandler>) {
        let format_id = handler.format_id().to_owned();
        self.handlers.insert(format_id, handler);
    }

    pub fn get(&self, format_id: &str) -> Result<Arc<dyn PackLifecycleHandler>, InstanceError> {
        self.handlers.get(format_id).cloned().ok_or_else(|| {
            InstanceError::UnsupportedOperation(format!(
                "No pack lifecycle handler registered for format '{format_id}'"
            ))
        })
    }

    pub fn list_format_ids(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}
