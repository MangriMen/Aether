use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverridableDto<T> {
    pub is_active: bool,
    pub data: T,
}
