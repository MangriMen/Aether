use std::{env, path::Path};

fn main() {
    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        let manifest = Path::new("app.manifest").canonicalize().unwrap();

        println!(
            "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
            manifest.to_str().unwrap()
        );

        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    }
}
