use std::hash::Hash;
use std::marker::PhantomData;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

// --- CacheKey & CacheId ---

#[derive(Debug)]
pub struct CacheKey<T> {
    pub namespace: &'static str,
    pub id: CacheId,
    _marker: PhantomData<T>,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum CacheId {
    Static(&'static str),
    Named(String),
}

impl AsRef<str> for CacheId {
    fn as_ref(&self) -> &str {
        match self {
            CacheId::Static(s) => s,
            CacheId::Named(s) => s.as_str(),
        }
    }
}

impl<T> CacheKey<T> {
    pub fn new(namespace: &'static str, id: CacheId) -> Self {
        Self {
            namespace,
            id,
            _marker: PhantomData,
        }
    }

    pub fn namespace(&self) -> &str {
        self.namespace
    }
    pub fn id(&self) -> &CacheId {
        &self.id
    }
}

// --- CachedValue ---

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

// --- AssetError ---

#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("Source read error: {0}")]
    ReadSource(String),
}
