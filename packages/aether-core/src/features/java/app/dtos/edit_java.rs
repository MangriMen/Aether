use crate::features::java::JavaPath;

#[derive(Debug)]
pub struct EditJava {
    pub major_version: u32,
    pub path: JavaPath,
}
