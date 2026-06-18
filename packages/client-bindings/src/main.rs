// Type generation binary for Aether's Tauri Specta bindings.
// Run via: cargo run -p client-bindings
//
// This is a dev-only tool isolated in its own crate so that
// `cargo tauri build` (which operates on apps/desktop/) never
// picks it up for bundling.

use aether_lib::shared::specta::{
    export_specta_builders, get_all_features_builders, get_export_path,
};

fn main() {
    let out_dir = get_export_path();

    export_specta_builders(&get_all_features_builders());

    println!("Type generation completed.");
    println!("Output directory: {}", out_dir.display());
}
