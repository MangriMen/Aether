use std::collections::HashMap;

use tauri_build::{DefaultPermissionRule, InlinedPlugin};

fn main() {
    let auth_plugin = (
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
            "list_importers",
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
            "list_content",
            "install_content",
            "enable_contents",
            "disable_contents",
            "remove_contents",
            "list_content_providers",
            "search_content",
            "check_compatibility",
            "get_content",
            "list_content_version",
        ]),
    );

    let minecraft_plugin = (
        "minecraft",
        InlinedPlugin::new().commands(&[
            "get_minecraft_version_manifest",
            "get_loader_version_manifest",
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
        auth_plugin,
        instance_plugin,
        minecraft_plugin,
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
