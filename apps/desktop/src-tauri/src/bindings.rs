#[cfg(debug_assertions)]
struct Exporter {
    out_dir: String,
}

#[cfg(debug_assertions)]
impl Exporter {
    fn new(out_dir: &str) -> Self {
        let _ = std::fs::create_dir_all(out_dir);
        Self {
            out_dir: out_dir.to_string(),
        }
    }

    fn export<F>(&self, plugin_name: &'static str, configure: F)
    where
        F: FnOnce(tauri_specta::Builder<tauri::Wry>) -> tauri_specta::Builder<tauri::Wry>,
    {
        let builder = tauri_specta::Builder::<tauri::Wry>::new()
            .error_handling(tauri_specta::ErrorHandlingMode::Throw)
            .plugin_name(plugin_name)
            .disable_serde_phases();

        let builder = configure(builder);

        builder
            .export(
                specta_typescript::Typescript::default(),
                format!("{}/{}.ts", self.out_dir, plugin_name),
            )
            .unwrap_or_else(|err| panic!("Failed to export {} bindings.{}", plugin_name, err));
    }
}

#[cfg(debug_assertions)]
pub fn generate_bindings() {
    use crate::commands::{
        APPLICATION_PLUGIN_NAME, AUTH_PLUGIN_NAME, EVENTS_PLUGIN_NAME, INSTANCE_PLUGIN_NAME,
        MINECRAFT_PLUGIN_NAME, PLUGIN_PLUGIN_NAME, PROCESS_PLUGIN_NAME, SETTINGS_PLUGIN_NAME,
    };

    let out_dir = "../src/6_shared/api/bindings";
    let exporter = Exporter::new(out_dir);

    exporter.export(APPLICATION_PLUGIN_NAME, |b| {
        b.commands(crate::core::app::get_specta_data())
    });

    exporter.export(AUTH_PLUGIN_NAME, |b| {
        b.commands(crate::features::auth::get_specta_data())
    });

    exporter.export(EVENTS_PLUGIN_NAME, |b| {
        use crate::features::events;
        b.commands(events::get_specta_data())
            .typ::<events::ProgressEventDto>()
    });

    exporter.export(INSTANCE_PLUGIN_NAME, |b| {
        b.commands(crate::features::instance::get_specta_data())
    });

    exporter.export(MINECRAFT_PLUGIN_NAME, |b| {
        b.commands(crate::features::minecraft::get_specta_data())
    });

    exporter.export(PLUGIN_PLUGIN_NAME, |b| {
        b.commands(crate::features::plugins::get_specta_data())
    });

    exporter.export(PROCESS_PLUGIN_NAME, |b| {
        b.commands(crate::features::process::get_specta_data())
    });

    exporter.export(SETTINGS_PLUGIN_NAME, |b| {
        b.commands(crate::features::settings::get_specta_data())
    });
}
