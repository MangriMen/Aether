#[derive(Debug)]
pub struct InstallJava {
    pub version: u32,
    pub force: bool,
}

impl InstallJava {
    pub fn new(version: u32) -> Self {
        Self {
            version,
            force: false,
        }
    }

    pub fn force(version: u32) -> Self {
        Self {
            version,
            force: true,
        }
    }
}
