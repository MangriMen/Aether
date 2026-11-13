//! The database driver's errors turned into the category the domain knows them by.

use super::error::DatabaseError;

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        Self::new(err)
    }
}
