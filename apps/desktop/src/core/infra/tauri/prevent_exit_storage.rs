use std::sync::atomic::{AtomicBool, Ordering};

pub struct PreventExitState(AtomicBool);

impl PreventExitState {
    pub fn new(state: bool) -> Self {
        Self(AtomicBool::new(state))
    }

    pub fn is_prevented(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    pub fn set_prevented(&self, value: bool) {
        self.0.store(value, Ordering::SeqCst);
    }
}
