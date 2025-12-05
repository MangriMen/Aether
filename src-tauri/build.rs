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
            "create",
            "import",
            "list",
            "get",
            "get_dir",
            "install",
            "update",
            "edit",
            "remove",
            "launch",
            "stop",
            "list_import_configs",
            "import_contents",
            "get_contents",
            "install_content",
            "enable_contents",
            "disable_contents",
            "remove_content",
            "remove_contents",
            "get_content_providers",
            "get_content_by_provider",
            "get_metadata_field_to_check_installed",
        ]),
    );

    let process_plugin = (
        "process",
        InlinedPlugin::new().commands(&["list", "get_by_instance_id"]),
    );

    let plugin_plugin = (
        "plugin",
        InlinedPlugin::new().commands(&[
            "import",
            "sync",
            "list",
            "get",
            "remove",
            "enable",
            "disable",
            "call",
            "get_settings",
            "edit_settings",
            "open_plugins_folder",
            "list_importers",
            "get_api_version",
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
            "get_app_settings",
            "edit_app_settings",
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
