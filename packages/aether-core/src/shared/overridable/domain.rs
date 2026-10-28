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
    use rstest::rstest;

    #[rstest]
    #[case::active_uses_data(true, "custom")]
    #[case::inactive_uses_global(false, "global")]
    fn should_resolve_to_the_matching_value_when_the_override_toggles(
        #[case] is_active: bool,
        #[case] expected: &str,
    ) {
        let overridable = Overridable::new(String::from("custom"), is_active);
        let result = overridable.resolve(&String::from("global"));
        assert_eq!(result, expected);
    }

    #[test]
    fn should_transform_data_when_mapped() {
        let overridable = Overridable::new(5, true);
        let mapped = overridable.map(|x| x * 2);
        assert_eq!(mapped.data, 10);
        assert!(mapped.is_active);
    }

    #[test]
    fn should_preserve_is_active_when_mapped() {
        let overridable = Overridable::new("hello", false);
        let mapped = overridable.map(|s| format!("{s}!"));
        assert!(!mapped.is_active);
    }

    #[test]
    fn should_create_an_inactive_default_when_built_as_new_disabled_default() {
        let overridable = Overridable::<i32>::new_disabled_default();
        assert!(!overridable.is_active);
        assert_eq!(overridable.data, 0);
    }

    #[test]
    fn should_roundtrip_through_serde_when_serialized_and_deserialized() {
        let overridable = Overridable::new(42, true);
        let json = serde_json::to_string(&overridable).unwrap();
        assert!(json.contains(r#""is_active":true"#), "snake_case: {json}");
        let deserialized: Overridable<i32> = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_active);
        assert_eq!(deserialized.data, 42);
    }
}
