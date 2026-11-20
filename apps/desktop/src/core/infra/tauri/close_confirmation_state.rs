use std::sync::atomic::{AtomicBool, Ordering};

/// Whether the frontend listens for the close confirmation. A close is held for it only then:
/// with nobody to ask the user, the window would never close.
#[derive(Default)]
pub struct CloseConfirmationState(AtomicBool);

impl CloseConfirmationState {
    pub fn is_ready(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    pub fn set_ready(&self, value: bool) {
        self.0.store(value, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_not_be_ready_until_the_frontend_says_so() {
        let state = CloseConfirmationState::default();
        assert!(!state.is_ready());

        state.set_ready(true);
        assert!(state.is_ready());

        state.set_ready(false);
        assert!(!state.is_ready());
    }
}
