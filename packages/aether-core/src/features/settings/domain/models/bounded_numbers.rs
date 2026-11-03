use std::num::{NonZeroU32, NonZeroUsize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("The number of concurrent downloads must be at least 1, got {0}")]
pub struct InvalidMaxConcurrentDownloads(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("The memory amount must be at least 1 MB, got {0}")]
pub struct InvalidMemoryMb(pub i64);

/// How many downloads may run at once. Always at least 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxConcurrentDownloads(NonZeroUsize);

impl MaxConcurrentDownloads {
    pub fn new(value: usize) -> Result<Self, InvalidMaxConcurrentDownloads> {
        NonZeroUsize::new(value)
            .map(Self)
            .ok_or_else(|| InvalidMaxConcurrentDownloads(i64::try_from(value).unwrap_or(i64::MAX)))
    }

    /// Lenient read of a stored value: an invalid one is logged and replaced by the default.
    pub(crate) fn from_stored_or_default(value: i64, context: &str) -> Self {
        Self::try_from(value).unwrap_or_else(|err| {
            tracing::warn!("Invalid stored value in {context}: {err}; using the default");
            Self::default()
        })
    }

    pub fn get(self) -> usize {
        self.0.get()
    }
}

impl Default for MaxConcurrentDownloads {
    fn default() -> Self {
        Self(NonZeroUsize::new(10).expect("10 is not zero"))
    }
}

impl TryFrom<i64> for MaxConcurrentDownloads {
    type Error = InvalidMaxConcurrentDownloads;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        usize::try_from(value)
            .ok()
            .and_then(NonZeroUsize::new)
            .map(Self)
            .ok_or(InvalidMaxConcurrentDownloads(value))
    }
}

impl From<MaxConcurrentDownloads> for usize {
    fn from(value: MaxConcurrentDownloads) -> Self {
        value.get()
    }
}

impl From<MaxConcurrentDownloads> for i64 {
    fn from(value: MaxConcurrentDownloads) -> Self {
        i64::try_from(value.get()).unwrap_or(i64::MAX)
    }
}

/// Memory amount in megabytes. Always at least 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryMb(NonZeroU32);

impl MemoryMb {
    pub fn new(value: u32) -> Result<Self, InvalidMemoryMb> {
        NonZeroU32::new(value)
            .map(Self)
            .ok_or(InvalidMemoryMb(i64::from(value)))
    }

    /// Lenient read of a stored value: an invalid one is logged and replaced by the default.
    pub(crate) fn from_stored_or_default(value: i64, context: &str) -> Self {
        Self::try_from(value).unwrap_or_else(|err| {
            tracing::warn!("Invalid stored value in {context}: {err}; using the default");
            Self::default()
        })
    }

    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl Default for MemoryMb {
    fn default() -> Self {
        Self(NonZeroU32::new(2048).expect("2048 is not zero"))
    }
}

impl TryFrom<i64> for MemoryMb {
    type Error = InvalidMemoryMb;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        u32::try_from(value)
            .ok()
            .and_then(NonZeroU32::new)
            .map(Self)
            .ok_or(InvalidMemoryMb(value))
    }
}

impl From<MemoryMb> for u32 {
    fn from(value: MemoryMb) -> Self {
        value.get()
    }
}

impl From<MemoryMb> for i64 {
    fn from(value: MemoryMb) -> Self {
        i64::from(value.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_reject_zero_downloads() {
        assert_eq!(
            MaxConcurrentDownloads::new(0),
            Err(InvalidMaxConcurrentDownloads(0))
        );
        assert_eq!(MaxConcurrentDownloads::new(3).unwrap().get(), 3);
    }

    #[test]
    fn should_default_downloads_to_ten() {
        assert_eq!(MaxConcurrentDownloads::default().get(), 10);
    }

    #[test]
    fn should_reject_non_positive_i64_downloads() {
        assert!(MaxConcurrentDownloads::try_from(0_i64).is_err());
        assert!(MaxConcurrentDownloads::try_from(-5_i64).is_err());
        assert_eq!(MaxConcurrentDownloads::try_from(7_i64).unwrap().get(), 7);
        assert_eq!(i64::from(MaxConcurrentDownloads::default()), 10);
    }

    #[test]
    fn should_reject_zero_memory() {
        assert_eq!(MemoryMb::new(0), Err(InvalidMemoryMb(0)));
        assert_eq!(MemoryMb::new(512).unwrap().get(), 512);
    }

    #[test]
    fn should_default_memory_to_2048() {
        assert_eq!(MemoryMb::default().get(), 2048);
    }

    #[test]
    fn should_reject_invalid_i64_memory() {
        assert!(MemoryMb::try_from(0_i64).is_err());
        assert!(MemoryMb::try_from(-1_i64).is_err());
        assert!(MemoryMb::try_from(i64::from(u32::MAX) + 1).is_err());
        assert_eq!(
            MemoryMb::try_from(i64::from(u32::MAX)).unwrap().get(),
            u32::MAX
        );
    }
}
