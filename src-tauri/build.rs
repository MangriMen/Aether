use std::collections::HashMap;

use tauri_build::{DefaultPermissionRule, InlinedPlugin};

fn main() {
    let account_plugin = (
        "auth",
        InlinedPlugin::new().commands(&[
            "get_accounts",
            "create_offline_account",
            "change_account",
            "logout",
        ]),
    );

    let instance_plugin = (
        "instance",
        InlinedPlugin::new().commands(&[
            "instance_create",
            "instance_install",
            "instance_update",
            "instance_list",
            "instance_get",
            "instance_edit",
            "instance_remove",
            "instance_get_contents",
            "instance_disable_contents",
            "instance_enable_contents",
            "instance_remove_content",
            "instance_remove_contents",
            "instance_launch",
            "instance_stop",
            "instance_import",
            "instance_get_import_configs",
            "instance_get_content_providers",
            "instance_get_content_by_provider",
            "instance_install_content",
            "instance_get_metadata_field_to_check_installed",
            "instance_import_contents",
            "instance_get_dir",
        ]),
    );

    let process_plugin = (
        "process",
        InlinedPlugin::new().commands(&["process_list", "process_get_by_instance_id"]),
    );

    let plugin_plugin = (
        "plugin",
        InlinedPlugin::new().commands(&[
            "sync_plugins",
            "list_plugins",
            "plugin_get",
            "is_plugin_enabled",
            "enable_plugin",
            "disable_plugin",
            "call_plugin",
            "plugin_get_settings",
            "plugin_edit_settings",
            "open_plugins_folder",
        ]),
    );

    let settings_plugin = (
        "settings",
        InlinedPlugin::new().commands(&[
            "get",
            "edit",
            "get_max_ram",
            "get_default_instance_settings",
            "edit_default_instance_settings",
        ]),
    );

    let mut plugins = HashMap::from([
        account_plugin,
        instance_plugin,
        process_plugin,
        plugin_plugin,
        settings_plugin,
    ]);

    let mut attributes = tauri_build::Attributes::new().codegen(tauri_build::CodegenContext::new());

    for (plugin_name, plugin) in plugins.iter_mut() {
        attributes = attributes.plugin(
            plugin_name,
            std::mem::take(plugin).default_permission(DefaultPermissionRule::AllowAllCommands),
        );
    }

    tauri_build::try_build(attributes).expect("Failed to run tauri-build");
}
