pub struct PreventExitState(std::sync::Mutex<PreventExitStorage>);

impl PreventExitState {
    pub fn new(state: bool) -> Self {
        Self(std::sync::Mutex::new(PreventExitStorage(state)))
    }

    pub fn is_prevented(&self) -> bool {
        self.0.lock().is_ok_and(|guard| guard.is_prevented())
    }

    pub fn set_prevented(&self, value: bool) {
        if let Ok(mut guard) = self.0.lock() {
            guard.set_prevented(value);
        }
    }
}

pub struct PreventExitStorage(bool);

impl PreventExitStorage {
    #[must_use]
    pub fn new(state: bool) -> Self {
        Self(state)
    }

    #[must_use]
    pub fn is_prevented(&self) -> bool {
        self.0
    }

    pub fn set_prevented(&mut self, value: bool) {
        self.0 = value;
    }
}
