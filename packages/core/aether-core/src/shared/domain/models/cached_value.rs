use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedValue<T> {
    pub value: T,
    pub updated_at: SystemTime,
}

impl<T> CachedValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            updated_at: SystemTime::now(),
        }
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        SystemTime::now() > self.updated_at + ttl
    }
}
