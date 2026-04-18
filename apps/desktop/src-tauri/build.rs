#[path = "src/commands.rs"]
mod commands;

use commands::*;

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
