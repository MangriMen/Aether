use crate::features::plugins::{
    PluginError, PluginRegistry, PluginSettings, PluginSettingsStorage,
};

/// What became of a request to give a plugin access to something — a folder, a host — as far as
/// its settings are concerned.
pub(super) enum SettingsGrant {
    /// The plugin already has the access: nothing changed.
    AlreadyAllowed,
    /// A dry run, and the plugin lacks the access: nothing changed.
    Needed,
    /// The access was added and saved.
    Granted,
}

/// The settings half of every grant. A dry run goes through the same checks as the grant, so
/// the UI asks the user only when something would change and never about what would be refused.
pub(super) async fn grant_in_settings(
    registry: &PluginRegistry,
    storage: &dyn PluginSettingsStorage,
    plugin_id: &str,
    dry_run: bool,
    is_allowed: impl FnOnce(&PluginSettings) -> bool,
    add: impl FnOnce(&mut PluginSettings),
) -> Result<SettingsGrant, PluginError> {
    let _settings_lock = registry.lock_settings(plugin_id).await;
    let mut settings = storage.get(plugin_id).await?.unwrap_or_default();

    if is_allowed(&settings) {
        return Ok(SettingsGrant::AlreadyAllowed);
    }

    if dry_run {
        return Ok(SettingsGrant::Needed);
    }

    add(&mut settings);
    storage.upsert(plugin_id, &settings).await?;

    Ok(SettingsGrant::Granted)
}

/// Takes back what a grant added when what had to follow it (a restart) failed. Only that: the
/// settings are read again, so a change saved in between stays.
pub(super) async fn revoke_in_settings(
    registry: &PluginRegistry,
    storage: &dyn PluginSettingsStorage,
    plugin_id: &str,
    remove: impl FnOnce(&mut PluginSettings),
) -> Result<(), PluginError> {
    let _settings_lock = registry.lock_settings(plugin_id).await;
    let Some(mut settings) = storage.get(plugin_id).await? else {
        return Ok(());
    };

    remove(&mut settings);
    storage.upsert(plugin_id, &settings).await
}
