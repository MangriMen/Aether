use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Overridable<T> {
    pub data: T,
    pub is_active: bool,
}

impl<T> Overridable<T> {
    pub fn new(data: T, is_active: bool) -> Self {
        Self { data, is_active }
    }
}

impl<T: Default> Overridable<T> {
    pub fn new_disabled_default() -> Self {
        Self::new(T::default(), false)
    }
}

impl<T: Clone> Overridable<T> {
    pub fn resolve<B>(&self, global: &B) -> T
    where
        B: ToOwned<Owned = T> + ?Sized,
    {
        if self.is_active {
            self.data.clone()
        } else {
            global.to_owned()
        }
    }
}
