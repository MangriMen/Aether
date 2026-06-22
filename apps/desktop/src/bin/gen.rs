// Type generation binary for Aether's Tauri Specta bindings.
// Run via: cargo gen
//
// Lives inside apps/desktop so it shares the same compilation profile
// as the main binary, avoiding double compilation of aether_lib in CI.

use std::path::PathBuf;

fn main() {
    // Determine and expose the export path before calling the export functions
    // which internally read BINDINGS_EXPORT_PATH via get_export_path().
    let export_path = std::env::var("BINDINGS_EXPORT_PATH").map_or_else(
        |_| {
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let workspace_root = manifest_dir.parent().unwrap();
            let default_path = workspace_root.join("frontend/src/shared/api/bindings");

            // SAFETY: single-threaded binary, no concurrent access to env
            unsafe { std::env::set_var("BINDINGS_EXPORT_PATH", &default_path) };

            default_path
        },
        PathBuf::from,
    );

    println!("Generating TypeScript bindings into: {}", export_path.display());

    let builders = aether_lib::shared::specta::get_all_features_builders();
    aether_lib::shared::specta::export_specta_builders(&builders);

    println!("Type generation completed.");
    println!("Output directory: {}", export_path.display());
}
