use std::hash::Hash;
use std::marker::PhantomData;

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
