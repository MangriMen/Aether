use super::ProgressEventType;

/// A download that every install needing the same thing waits on. The one key both the bar that
/// starts it and the bars that wait on it use, so they cannot disagree on what is shared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharedDownload {
    Java { version: u32 },
    MinecraftVersion { version: String },
}

impl SharedDownload {
    /// What the shared bar of this download reports.
    pub fn event_type(&self) -> ProgressEventType {
        match self {
            Self::Java { version } => ProgressEventType::JavaDownload { version: *version },
            Self::MinecraftVersion { version } => ProgressEventType::MinecraftVersionDownload {
                version: version.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_is_the_one_of_the_download() {
        assert_eq!(
            SharedDownload::Java { version: 21 }.event_type(),
            ProgressEventType::JavaDownload { version: 21 }
        );
        assert_eq!(
            SharedDownload::MinecraftVersion {
                version: "1.21".to_string()
            }
            .event_type(),
            ProgressEventType::MinecraftVersionDownload {
                version: "1.21".to_string()
            }
        );
    }
}
