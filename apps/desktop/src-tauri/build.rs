#[path = "src/shared/commands.rs"]
mod commands;

use commands::{
    APPLICATION_PLUGIN_NAME, AUTH_PLUGIN_NAME, EVENTS_PLUGIN_NAME, INSTANCE_PLUGIN_NAME,
    MINECRAFT_PLUGIN_NAME, PLUGIN_PLUGIN_NAME, PROCESS_PLUGIN_NAME, SETTINGS_PLUGIN_NAME,
    UPDATE_PLUGIN_NAME, application_commands, auth_commands, events_commands, instance_commands,
    minecraft_commands, plugin_commands, process_commands, settings_commands, update_commands,
};

macro_rules! to_tokens {
    ($($cmd:ident),* $(,)?) => {
        &[ $(stringify!($cmd)),* ]
    };
}

macro_rules! make_plugin_meta {
    ($($name:ident),* $(,)?) => {
        [
            $(
                PluginMeta {
                    name: paste::paste! { [< $name:upper _PLUGIN_NAME >] },
                    commands: paste::paste! { [< $name _commands >]!(to_tokens!) },
                }
            ),*
        ]
    };
}

struct PluginMeta {
    name: &'static str,
    commands: &'static [&'static str],
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/commands.rs");
    println!("cargo:rerun-if-changed=tauri.conf.json");

    let plugins = make_plugin_meta![
        application,
        auth,
        events,
        instance,
        minecraft,
        process,
        plugin,
        settings,
        update
    ];

    let mut attributes = tauri_build::Attributes::new().codegen(tauri_build::CodegenContext::new());

    for plugin in plugins {
        attributes = attributes.plugin(
            plugin.name,
            tauri_build::InlinedPlugin::new()
                .commands(plugin.commands)
                .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
        );
    }

    tauri_build::try_build(attributes).expect("Failed to run tauri-build");
}
