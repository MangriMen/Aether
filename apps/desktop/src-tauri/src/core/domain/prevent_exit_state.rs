pub type PreventExitState = std::sync::Mutex<PreventExitStateInner>;

pub struct PreventExitStateInner(bool);

impl PreventExitStateInner {
    #[must_use]
    pub fn new(state: bool) -> Self {
        Self(state)
    }

    #[must_use]
    pub fn is_prevented(&self) -> bool {
        self.0
    }

    pub fn set(&mut self, value: bool) {
        self.0 = value;
    }
}
