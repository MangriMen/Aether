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

    pub fn map<U, F>(self, f: F) -> Overridable<U>
    where
        F: FnOnce(T) -> U,
    {
        Overridable {
            data: f(self.data),
            is_active: self.is_active,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_uses_data_when_active() {
        let overridable = Overridable::new(String::from("custom"), true);
        let result = overridable.resolve(&String::from("global"));
        assert_eq!(result, "custom");
    }

    #[test]
    fn resolve_uses_global_when_not_active() {
        let overridable = Overridable::new(String::from("custom"), false);
        let result = overridable.resolve(&String::from("global"));
        assert_eq!(result, "global");
    }

    #[test]
    fn resolve_with_integer() {
        let overridable = Overridable::new(42, true);
        assert_eq!(overridable.resolve(&0), 42);

        let overridable = Overridable::new(42, false);
        assert_eq!(overridable.resolve(&100), 100);
    }

    #[test]
    fn map_transforms_data() {
        let overridable = Overridable::new(5, true);
        let mapped = overridable.map(|x| x * 2);
        assert_eq!(mapped.data, 10);
        assert!(mapped.is_active);
    }

    #[test]
    fn map_preserves_is_active() {
        let overridable = Overridable::new("hello", false);
        let mapped = overridable.map(|s| format!("{s}!"));
        assert!(!mapped.is_active);
    }

    #[test]
    fn new_disabled_default_creates_inactive_default() {
        let overridable = Overridable::<i32>::new_disabled_default();
        assert!(!overridable.is_active);
        assert_eq!(overridable.data, 0);
    }

    #[test]
    fn serde_roundtrip() {
        let overridable = Overridable::new(42, true);
        let json = serde_json::to_string(&overridable).unwrap();
        assert!(json.contains(r#""is_active":true"#), "snake_case: {json}");
        let deserialized: Overridable<i32> = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_active);
        assert_eq!(deserialized.data, 42);
    }
}
