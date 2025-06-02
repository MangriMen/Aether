pub struct PreventExitOnCloseStateInner(pub bool);
pub type PreventExitOnCloseState = std::sync::Mutex<PreventExitOnCloseStateInner>;
