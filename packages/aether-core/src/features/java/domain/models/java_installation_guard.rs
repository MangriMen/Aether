pub struct JavaInstallationGuard {
    on_drop: Option<Box<dyn FnOnce() + Send>>,
}

impl JavaInstallationGuard {
    pub fn new<F: FnOnce() + Send + 'static>(f: F) -> Self {
        Self {
            on_drop: Some(Box::new(f)),
        }
    }
}

impl Drop for JavaInstallationGuard {
    fn drop(&mut self) {
        if let Some(callback) = self.on_drop.take() {
            callback();
        }
    }
}
