use std::path::PathBuf;

/// Where a broken instance was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrokenInstanceOrigin {
    /// A row of the `instances` table that cannot be turned into an instance.
    Database,
    /// A folder with a legacy `instance.json` that is not in the table.
    LegacyFile,
}

/// Why an instance cannot be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BrokenInstanceReason {
    /// The id is not a plain directory name, so there is no folder it could name.
    #[error("the id is not a valid directory name")]
    InvalidId,

    #[error("invalid game_version \"{value}\"")]
    InvalidGameVersion { value: String },

    #[error("invalid mod loader \"{value}\"")]
    InvalidLoader { value: String },

    #[error("invalid install stage \"{value}\"")]
    InvalidInstallStage { value: String },

    /// Any other column (JSON columns, timestamps) whose value cannot be read.
    #[error("invalid {column}")]
    InvalidColumn { column: String },

    /// The file of a legacy instance cannot be read or parsed.
    #[error("unreadable instance file: {details}")]
    UnreadableFile { details: String },

    /// A legacy instance file in a folder of another name: whose instance it is, is not clear,
    /// and the launcher does not take it in.
    #[error("the file belongs to the instance \"{file_id}\"")]
    LegacyIdMismatch { file_id: String },

    /// A legacy instance file that reads fine but was not moved into the database.
    #[error("not moved into the database")]
    NotMigrated,
}

/// An instance that is stored but cannot be read, shown so that the user can see it and get
/// rid of it instead of it just disappearing from the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenInstance {
    /// The raw id: the text of the row, or the name of the folder.
    pub id: String,
    /// The folder of the instance; only for an id that names one.
    pub dir: Option<PathBuf>,
    pub origin: BrokenInstanceOrigin,
    pub reason: BrokenInstanceReason,
}

/// A stored record that cannot be read, as the storage reports it: the folder it names is for
/// whoever knows where instances live to work out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokenRecord {
    pub id: String,
    pub reason: BrokenInstanceReason,
}

/// How the file system sees an id: Windows takes no notice of case, nor of dots and spaces at
/// the end of a name, so `Foo.` is the folder `foo`.
pub fn folder_key(id: &str) -> String {
    id.trim_end_matches(['.', ' ']).to_lowercase()
}
