use aether_lib::shared::specta::{
    export_specta_builders, get_all_features_builders, get_export_path,
};

fn main() {
    let out_dir = get_export_path();

    export_specta_builders(&get_all_features_builders());

    println!("Type generation completed.");
    println!("Output directory: {}", out_dir.display());
}
