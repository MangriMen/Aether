// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(e) = aether::core::launch_app() {
        eprintln!("Failed to launch app: {e}")
    }
}
