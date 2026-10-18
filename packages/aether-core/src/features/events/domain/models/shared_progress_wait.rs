/// A bar's wait on a shared download, from
/// [`ProgressService::wait_on_shared`](crate::features::events::ProgressService::wait_on_shared):
/// while it lives, the bar gets a mirror of every shared bar of that download that starts.
/// Dropping it ends the wait; a mirror the bar already has still ends with its shared bar.
#[must_use = "the wait ends as soon as the guard is dropped"]
pub struct SharedProgressWait {
    on_drop: Option<Box<dyn FnOnce() + Send>>,
}

impl SharedProgressWait {
    pub fn new(on_drop: impl FnOnce() + Send + 'static) -> Self {
        Self {
            on_drop: Some(Box::new(on_drop)),
        }
    }

    /// A wait that tracks nothing: for a service that keeps no mirrors.
    pub fn none() -> Self {
        Self { on_drop: None }
    }
}

impl Drop for SharedProgressWait {
    fn drop(&mut self) {
        if let Some(on_drop) = self.on_drop.take() {
            on_drop();
        }
    }
}
