use crate::features::instance::{ContentCompatibilityCheckParams, Instance};

#[derive(Debug, Clone)]
pub struct PluginCheckCompatibilityParams {
    pub instances: Vec<Instance>,
    pub check_params: ContentCompatibilityCheckParams,
}
