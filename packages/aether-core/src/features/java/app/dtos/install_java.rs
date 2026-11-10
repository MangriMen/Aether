#[derive(Debug)]
pub struct InstallJava {
    pub version: u32,
    pub force: bool,
    /// Whether the user can call the install off from its own bar: only for an install they
    /// started, not for one that is a part of another.
    pub cancellable: bool,
}

impl InstallJava {
    pub fn new(version: u32) -> Self {
        Self {
            version,
            force: false,
            cancellable: false,
        }
    }

    pub fn force(version: u32) -> Self {
        Self {
            version,
            force: true,
            cancellable: false,
        }
    }
}
