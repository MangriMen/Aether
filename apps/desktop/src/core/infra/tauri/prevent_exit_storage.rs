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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prevent_exit_state_starts_true() {
        let state = PreventExitState::new(true);
        assert!(state.is_prevented());
    }

    #[test]
    fn prevent_exit_state_starts_false() {
        let state = PreventExitState::new(false);
        assert!(!state.is_prevented());
    }

    #[test]
    fn prevent_exit_state_toggle_to_true() {
        let state = PreventExitState::new(false);
        state.set_prevented(true);
        assert!(state.is_prevented());
    }

    #[test]
    fn prevent_exit_state_toggle_to_false() {
        let state = PreventExitState::new(true);
        state.set_prevented(false);
        assert!(!state.is_prevented());
    }

    #[test]
    fn prevent_exit_state_set_twice() {
        let state = PreventExitState::new(false);
        state.set_prevented(true);
        state.set_prevented(false);
        assert!(!state.is_prevented());
    }
}
