#[cfg(debug_assertions)]
macro_rules! generate_plugin_bindings {
    ($out_dir:expr, $($name:ident),* $(,)?) => {
        $(
            tauri_specta::Builder::<tauri::Wry>::new()
                .error_handling(tauri_specta::ErrorHandlingMode::Throw)
                .plugin_name(paste::paste! { crate::commands::[< $name:upper _PLUGIN_NAME >] })
                .commands(crate::features::$name::get_specta_data())
                .export(
                    specta_typescript::Typescript::default(),
                    format!("{}/{}.ts", $out_dir, stringify!($name)),
                )
                .expect(&format!("Failed to export {} bindings", stringify!($name)));
        )*
    };
}

#[cfg(debug_assertions)]
pub fn generate_bindings() {
    let out_dir = "../src/6_shared/api/bindings";

    let _ = std::fs::create_dir_all(out_dir);

    generate_plugin_bindings![
        out_dir, auth, //  instance,  process, plugin,
        minecraft, settings
    ];
}

// #[cfg(test)]
// mod tests {
//     use crate::bindings::generate_bindings;

//     #[test]
//     fn generate_bindings_specta() {
//         generate_bindings();
//     }
// }
