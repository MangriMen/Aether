use aether_core::shared::overridable::Overridable;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub struct OverridableDto<T> {
    pub is_active: bool,
    pub data: T,
}

impl<T, U> From<Overridable<T>> for OverridableDto<U>
where
    T: Into<U>,
{
    fn from(value: Overridable<T>) -> Self {
        Self {
            is_active: value.is_active,
            data: value.data.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::inactive(false, 42)]
    #[case::active(true, 100)]
    fn overridable_dto_from(#[case] is_active: bool, #[case] data: i32) {
        let overridable = Overridable::<i32> { is_active, data };
        let dto: OverridableDto<i32> = overridable.into();
        assert_eq!(dto.is_active, is_active);
        assert_eq!(dto.data, data);
    }

    #[test]
    fn overridable_dto_from_with_conversion() {
        let overridable = Overridable::<&str> {
            is_active: true,
            data: "hello",
        };
        let dto: OverridableDto<String> = overridable.into();
        assert!(dto.is_active);
        assert_eq!(dto.data, "hello");
    }

    #[test]
    fn overridable_dto_serialize_camel_case() {
        let dto = OverridableDto::<i32> {
            is_active: true,
            data: 7,
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("isActive"), "Expected camelCase, got: {json}");
        assert!(
            !json.contains("is_active"),
            "Expected camelCase, got: {json}"
        );
    }

    #[test]
    fn overridable_dto_deserialize_camel_case() {
        let json = r#"{"isActive": false, "data": 99}"#;
        let dto: OverridableDto<i32> = serde_json::from_str(json).unwrap();
        assert!(!dto.is_active);
        assert_eq!(dto.data, 99);
    }
}
