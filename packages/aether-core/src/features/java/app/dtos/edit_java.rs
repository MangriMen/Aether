use std::path::PathBuf;

#[derive(Debug)]
pub struct EditJava {
    pub major_version: u32,
    pub path: PathBuf,
}
