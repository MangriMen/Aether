use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;

use crate::{
    features::{
        instance::PluginId,
        plugins::{PluginError, PluginSettings, PluginSettingsStorage},
        settings::LocationInfo,
    },
    shared::io::infra::{read_toml_async, write_toml_async},
};

use super::PluginSettingsV1;

pub struct FsPluginSettingsStorage {
    location_info: Arc<LocationInfo>,
}

impl FsPluginSettingsStorage {
    pub async fn read<T>(&self, path: &Path) -> Result<Option<T>, PluginError>
    where
        T: serde::de::DeserializeOwned,
    {
        if !path.exists() {
            return Ok(None);
        }

        let value = read_toml_async::<T>(path).await?;

        Ok(Some(value))
    }

    pub async fn write<T>(&self, path: &Path, value: &T) -> Result<(), PluginError>
    where
        T: serde::Serialize,
    {
        Ok(write_toml_async(path, value).await?)
    }
}

impl FsPluginSettingsStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn get_plugin_settings_path(&self, plugin_id: &PluginId) -> PathBuf {
        self.location_info.plugin_settings(plugin_id)
    }
}

#[async_trait]
impl PluginSettingsStorage for FsPluginSettingsStorage {
    async fn get(&self, plugin_id: &PluginId) -> Result<Option<PluginSettings>, PluginError> {
        let dto: Option<PluginSettingsV1> =
            self.read(&self.get_plugin_settings_path(plugin_id)).await?;
        Ok(dto.map(PluginSettings::from))
    }

    async fn upsert(
        &self,
        plugin_id: &PluginId,
        settings: &PluginSettings,
    ) -> Result<(), PluginError> {
        let dto = PluginSettingsV1::from(settings.clone());
        self.write(&self.get_plugin_settings_path(plugin_id), &dto)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::plugins::{CommandApproval, CommandIdentity, partition_mountable};

    #[tokio::test]
    async fn should_keep_a_bad_folder_in_the_file_when_an_unrelated_upsert_is_saved() {
        let root = tempfile::tempdir().unwrap();
        let storage = FsPluginSettingsStorage::new(Arc::new(LocationInfo::new(
            root.path().join("settings"),
            root.path().join("config"),
        )));
        let plugin_id = PluginId::from_static("packwiz");
        let file = storage.get_plugin_settings_path(&plugin_id);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        let good = root.path().join("good");
        std::fs::create_dir_all(&good).unwrap();
        std::fs::write(
            &file,
            format!(
                "allowedHosts = []\nallowedPaths = [\n  {{ host = {good:?}, plugin = \"/good\" }},\n  \
                 {{ host = \"/home/u/a\", plugin = \"/cache\", readOnly = false }},\n]\n"
            ),
        )
        .unwrap();

        let mut settings = storage.get(&plugin_id).await.unwrap().unwrap();
        settings.allowed_hosts.push("example.com".to_owned());
        storage.upsert(&plugin_id, &settings).await.unwrap();

        let reread = storage.get(&plugin_id).await.unwrap().unwrap();
        assert_eq!(reread.allowed_hosts, ["example.com"]);
        assert_eq!(reread.allowed_paths.len(), 1);
        assert_eq!(reread.ignored_paths.len(), 1);
        assert_eq!(reread.ignored_paths[0].path.host, "/home/u/a");
        assert_eq!(reread.ignored_paths[0].path.plugin, "/cache");
        assert!(!reread.ignored_paths[0].path.read_only);

        // The ignored entry is kept, not granted: only the valid folder is mounted.
        let (mountable, _) = partition_mountable(&reread.allowed_paths, &[]);
        assert_eq!(mountable.len(), 1);
        assert_eq!(mountable[0].plugin.as_str(), "/good");
    }

    #[tokio::test]
    async fn should_keep_command_approvals_when_the_settings_are_saved_and_read_again() {
        let root = tempfile::tempdir().unwrap();
        let storage = FsPluginSettingsStorage::new(Arc::new(LocationInfo::new(
            root.path().join("settings"),
            root.path().join("config"),
        )));
        let plugin_id = PluginId::from_static("packwiz");
        let mut settings = PluginSettings::default();
        settings.command_approvals.push(CommandApproval {
            identity: CommandIdentity {
                jar_sha256: "ab".repeat(32),
                jvm_options: vec!["-Xmx1G".to_owned()],
            },
            plugin_sha256: "cd".repeat(32),
            jar: "/cache/installer.jar".to_owned(),
            approved_at: chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        });

        storage.upsert(&plugin_id, &settings).await.unwrap();
        let reread = storage.get(&plugin_id).await.unwrap().unwrap();

        assert_eq!(reread.command_approvals, settings.command_approvals);
        let text = std::fs::read_to_string(storage.get_plugin_settings_path(&plugin_id)).unwrap();
        assert!(text.contains("[[commandApprovals]]"), "{text}");
    }

    #[tokio::test]
    async fn should_not_write_a_command_approvals_table_when_there_are_none() {
        let root = tempfile::tempdir().unwrap();
        let storage = FsPluginSettingsStorage::new(Arc::new(LocationInfo::new(
            root.path().join("settings"),
            root.path().join("config"),
        )));
        let plugin_id = PluginId::from_static("packwiz");

        storage
            .upsert(&plugin_id, &PluginSettings::default())
            .await
            .unwrap();

        let text = std::fs::read_to_string(storage.get_plugin_settings_path(&plugin_id)).unwrap();
        assert!(!text.contains("commandApprovals"), "{text}");
        assert!(!text.contains("localHosts"), "{text}");
    }

    #[tokio::test]
    async fn should_keep_local_hosts_and_drop_one_it_did_not_write_when_read_again() {
        let root = tempfile::tempdir().unwrap();
        let storage = FsPluginSettingsStorage::new(Arc::new(LocationInfo::new(
            root.path().join("settings"),
            root.path().join("config"),
        )));
        let plugin_id = PluginId::from_static("packwiz");
        let file = storage.get_plugin_settings_path(&plugin_id);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(
            &file,
            "allowedHosts = []\nallowedPaths = []\n\
             localHosts = [\"192.168.1.10:8080\", \"*.lan\", \"[::1]\", \"8.8.8.8\"]\n",
        )
        .unwrap();

        let settings = storage.get(&plugin_id).await.unwrap().unwrap();
        let local_hosts: Vec<String> = settings
            .local_hosts
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(local_hosts, ["192.168.1.10:8080", "[::1]"]);

        storage.upsert(&plugin_id, &settings).await.unwrap();
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(
            text.contains(r#"localHosts = ["192.168.1.10:8080", "[::1]"]"#),
            "{text}"
        );
    }
}
