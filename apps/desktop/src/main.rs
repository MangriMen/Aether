// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
fn attach_to_parent_console() {
    use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};

    // Attach to the console of the parent process if it exists
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

fn main() {
    #[cfg(target_os = "windows")]
    attach_to_parent_console();

    if let Err(e) = aether_lib::core::launch_app() {
        eprintln!("Failed to launch app: {e}");
    }
}
