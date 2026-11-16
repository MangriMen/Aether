use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum PluginInternalEventDto {
    Loaded,
    Unloaded,
    BeforeInstanceLaunch { instance_id: String },
    AfterInstanceLaunch { instance_id: String },
}
