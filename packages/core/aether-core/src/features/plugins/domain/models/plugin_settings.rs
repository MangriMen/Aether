use super::PathMapping;

#[derive(Debug, Default, Clone)]
pub struct PluginSettings {
    pub allowed_hosts: Vec<String>,
    pub allowed_paths: Vec<PathMapping>,
}
