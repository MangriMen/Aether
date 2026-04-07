pub type PreventExitState = std::sync::Mutex<PreventExitStateInner>;

pub struct PreventExitStateInner(bool);

impl PreventExitStateInner {
    pub fn new(state: bool) -> Self {
        Self(state)
    }

    pub fn is_prevented(&self) -> bool {
        self.0
    }

    pub fn set(&mut self, value: bool) {
        self.0 = value;
    }
}
