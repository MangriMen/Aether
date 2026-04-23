use std::path::{Path, PathBuf};

pub struct Exporter {
    plugin_name: &'static str,
    pub builder: tauri_specta::Builder<tauri::Wry>,
    out_dir: PathBuf,
}

impl Exporter {
    fn new<T: AsRef<Path>>(
        plugin_name: &'static str,
        builder: tauri_specta::Builder<tauri::Wry>,
        out_dir: T,
    ) -> Self {
        let _ = std::fs::create_dir_all(&out_dir);
        Self {
            plugin_name,
            builder,
            out_dir: out_dir.as_ref().to_path_buf(),
        }
    }

    pub fn export(&self) {
        self.builder
            .export(
                specta_typescript::Typescript::default(),
                self.out_dir.join(format!("{}.ts", self.plugin_name)),
            )
            .unwrap_or_else(|err| panic!("Failed to export {} bindings.{}", self.plugin_name, err));
    }
}

fn get_default_builder(plugin_name: &'static str) -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        .error_handling(tauri_specta::ErrorHandlingMode::Throw)
        .plugin_name(plugin_name)
        .disable_serde_phases()
}

fn get_all_features_builders(out_dir: &PathBuf) -> Vec<Exporter> {
    use crate::commands::{
        APPLICATION_PLUGIN_NAME, AUTH_PLUGIN_NAME, EVENTS_PLUGIN_NAME, INSTANCE_PLUGIN_NAME,
        MINECRAFT_PLUGIN_NAME, PLUGIN_PLUGIN_NAME, PROCESS_PLUGIN_NAME, SETTINGS_PLUGIN_NAME,
        UPDATE_PLUGIN_NAME,
    };

    use crate::features::{auth, events, instance, minecraft, plugins, process, settings, update};

    let features_data: Vec<(
        &'static str,
        tauri_specta::Commands<tauri::Wry>,
        Option<tauri_specta::Events>,
    )> = vec![
        (
            APPLICATION_PLUGIN_NAME,
            crate::core::get_specta_commands(),
            None,
        ),
        (AUTH_PLUGIN_NAME, auth::get_specta_commands(), None),
        (
            EVENTS_PLUGIN_NAME,
            events::get_specta_commands(),
            Some(events::get_specta_events()),
        ),
        (
            INSTANCE_PLUGIN_NAME,
            instance::get_specta_commands(),
            Some(instance::get_specta_events()),
        ),
        (
            MINECRAFT_PLUGIN_NAME,
            minecraft::get_specta_commands(),
            None,
        ),
        (
            PLUGIN_PLUGIN_NAME,
            plugins::get_specta_commands(),
            Some(plugins::get_specta_events()),
        ),
        (
            PROCESS_PLUGIN_NAME,
            process::get_specta_commands(),
            Some(process::get_specta_events()),
        ),
        (SETTINGS_PLUGIN_NAME, settings::get_specta_commands(), None),
        (
            UPDATE_PLUGIN_NAME,
            update::get_specta_commands(),
            Some(update::get_specta_events()),
        ),
    ];

    features_data
        .into_iter()
        .map(|(name, commands, events)| {
            let builder = get_default_builder(name).commands(commands);

            let builder = if let Some(events) = events {
                builder.events(events)
            } else {
                builder
            };

            Exporter::new(name, builder, out_dir)
        })
        .collect()
}

pub fn create_specta_exporters() -> Vec<Exporter> {
    let out_dir = std::env::var("TYPE_GEN_EXPORT_DIR")
        .map(PathBuf::from)
        .expect("TYPE_GEN_EXPORT_DIR not specified");

    get_all_features_builders(&out_dir)
}
