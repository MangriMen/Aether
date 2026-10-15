use std::path::PathBuf;

pub enum FileEventKind {
    Create,
    Modify,
    Remove,
}

pub struct FileEvent {
    pub kind: FileEventKind,
    pub path: PathBuf,
}
