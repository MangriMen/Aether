use std::fmt;

/// The most results a provider is asked for in one page. Modrinth's search takes `limit` up to
/// 100 and refuses more.
pub const PAGE_SIZE_MAX: u32 = 100;

/// Which pagination value was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaginationField {
    Page,
    PageSize,
}

impl fmt::Display for PaginationField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Page => "page",
            Self::PageSize => "page size",
        })
    }
}

/// Why a number is not a page index or a page size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PaginationRejection {
    #[error("it must be at least 1")]
    TooSmall,
    #[error("it must be at most {0}")]
    TooLarge(u32),
}

/// A number refused by [`PageIndex::parse`] or [`PageSize::parse`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Invalid {field} {value}: {reason}")]
pub struct InvalidPagination {
    pub field: PaginationField,
    pub value: i64,
    pub reason: PaginationRejection,
}

/// A 1-based page of search results: the first page is 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PageIndex(u32);

impl PageIndex {
    pub const FIRST: Self = Self(1);

    pub fn parse(value: i64) -> Result<Self, InvalidPagination> {
        let reject = |reason| InvalidPagination {
            field: PaginationField::Page,
            value,
            reason,
        };
        if value < 1 {
            return Err(reject(PaginationRejection::TooSmall));
        }
        u32::try_from(value)
            .map(Self)
            .map_err(|_| reject(PaginationRejection::TooLarge(u32::MAX)))
    }

    pub fn get(self) -> u32 {
        self.0
    }

    /// How many results come before this page when pages hold `size` results.
    pub fn offset(self, size: PageSize) -> u64 {
        u64::from(self.0 - 1) * u64::from(size.0)
    }
}

impl TryFrom<i64> for PageIndex {
    type Error = InvalidPagination;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<PageIndex> for i64 {
    fn from(value: PageIndex) -> Self {
        i64::from(value.0)
    }
}

/// How many results a page holds: from 1 to [`PAGE_SIZE_MAX`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PageSize(u32);

impl PageSize {
    pub fn parse(value: i64) -> Result<Self, InvalidPagination> {
        let reject = |reason| InvalidPagination {
            field: PaginationField::PageSize,
            value,
            reason,
        };
        if value < 1 {
            return Err(reject(PaginationRejection::TooSmall));
        }
        match u32::try_from(value) {
            Ok(size) if size <= PAGE_SIZE_MAX => Ok(Self(size)),
            _ => Err(reject(PaginationRejection::TooLarge(PAGE_SIZE_MAX))),
        }
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl TryFrom<i64> for PageSize {
    type Error = InvalidPagination;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<PageSize> for i64 {
    fn from(value: PageSize) -> Self {
        i64::from(value.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_refuse_the_page_when_it_is_zero_or_negative() {
        for value in [0, -1, i64::MIN] {
            let err = PageIndex::parse(value).unwrap_err();
            assert_eq!(err.reason, PaginationRejection::TooSmall);
            assert_eq!(err.field, PaginationField::Page);
        }
    }

    #[test]
    fn should_refuse_the_page_when_it_does_not_fit_in_u32() {
        assert!(PageIndex::parse(i64::from(u32::MAX) + 1).is_err());
    }

    #[test]
    fn should_accept_the_page_when_it_is_positive() {
        assert_eq!(PageIndex::parse(1).unwrap(), PageIndex::FIRST);
        assert_eq!(PageIndex::parse(37).unwrap().get(), 37);
    }

    #[test]
    fn should_refuse_the_page_size_when_it_is_zero_or_negative() {
        for value in [0, -20] {
            let err = PageSize::parse(value).unwrap_err();
            assert_eq!(err.reason, PaginationRejection::TooSmall);
            assert_eq!(err.field, PaginationField::PageSize);
        }
    }

    #[test]
    fn should_refuse_the_page_size_when_it_is_over_the_cap() {
        for value in [i64::from(PAGE_SIZE_MAX) + 1, i64::MAX] {
            let err = PageSize::parse(value).unwrap_err();
            assert_eq!(err.reason, PaginationRejection::TooLarge(PAGE_SIZE_MAX));
        }
    }

    #[test]
    fn should_accept_the_page_size_when_it_is_within_the_cap() {
        assert_eq!(PageSize::parse(1).unwrap().get(), 1);
        assert_eq!(PageSize::parse(20).unwrap().get(), 20);
        assert_eq!(PageSize::parse(100).unwrap().get(), PAGE_SIZE_MAX);
    }
}
