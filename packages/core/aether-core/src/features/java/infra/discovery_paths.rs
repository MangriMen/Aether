use std::env;
use std::path::{Path, PathBuf};

/// Returns a consolidated list of default Java installation paths across different OS environments.
pub fn get_default_discovery_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // 1. Core Environment Paths
    if let Ok(java_home) = env::var("JAVA_HOME") {
        let p = PathBuf::from(java_home);
        if let Some(parent) = p.parent() {
            paths.push(parent.to_path_buf());
        }
        paths.push(p);
    }

    if let Ok(path_env) = env::var("PATH") {
        for split_path in env::split_paths(&path_env) {
            if split_path.file_name().is_some_and(|name| name == "bin")
                && let Some(parent) = split_path.parent()
            {
                paths.push(parent.to_path_buf());
            }
        }
    }

    // 2. Platform-Specific System and Launcher Paths
    paths.extend(get_os_specific_paths());

    // 3. Post-processing
    paths.retain(|p| p.exists() && p.is_dir());
    paths.sort();
    paths.dedup();

    paths
}

#[cfg(target_os = "windows")]
fn get_os_specific_paths() -> Vec<PathBuf> {
    let program_files =
        env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
    let program_files_x86 =
        env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
    let local_app_data = env::var("LocalAppData")
        .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local".to_string());
    let app_data =
        env::var("AppData").unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".to_string());

    let pf = Path::new(&program_files);
    let pf86 = Path::new(&program_files_x86);
    let local = Path::new(&local_app_data);
    let roaming = Path::new(&app_data);

    vec![
        // System / Vendor Runtimes
        pf.join("Java"),
        pf.join("Eclipse Foundation"),
        pf.join("Eclipse Adoptium"),
        pf.join("Amazon Corretto"),
        pf.join("Azul Systems"),
        pf.join("BellSoft"),
        pf.join("RedHat"),
        pf86.join("Java"),
        pf.join("JetBrains"),
        local.join("JetBrains").join("Toolbox").join("apps"),
        local.join("scoop").join("apps"),
        // App Stores & Launchers
        local
            .join("Packages")
            .join("Microsoft.42948320D69E7_8wekyb3d8bbwe")
            .join("LocalCache")
            .join("Local")
            .join("runtime"),
        roaming.join(".minecraft").join("runtime"),
        roaming.join("PrismLauncher").join("java"),
        roaming
            .join("ModrinthApp")
            .join("meta")
            .join("java_versions"),
    ]
}

#[cfg(target_os = "macos")]
fn get_os_specific_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        // System Runtimes
        PathBuf::from("/Library/Java/JavaVirtualMachines"),
        PathBuf::from("/System/Library/Java/JavaVirtualMachines"),
        // Package Managers
        PathBuf::from("/opt/homebrew/opt"),
        PathBuf::from("/usr/local/opt"),
        PathBuf::from("/opt/homebrew/Cellar"),
        PathBuf::from("/usr/local/Cellar"),
    ];

    if let Ok(home_env) = env::var("HOME") {
        let home = Path::new(&home_env);
        let app_support = home.join("Library").join("Application Support");

        paths.push(home.join(".sdkman").join("candidates").join("java"));
        paths.push(app_support.join("minecraft").join("runtime"));
        paths.push(app_support.join("PrismLauncher").join("java"));

        // ModrinthApp on macOS stores data directly inside its Application Support directory
        paths.push(
            app_support
                .join("ModrinthApp")
                .join("meta")
                .join("java_versions"),
        );
    }

    paths
}

#[cfg(target_os = "linux")]
fn get_os_specific_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        // System Package Managers
        PathBuf::from("/usr/lib/jvm"),
        PathBuf::from("/usr/lib64/jvm"),
        PathBuf::from("/usr/java"),
    ];

    if let Ok(home_env) = env::var("HOME") {
        let home = Path::new(&home_env);

        // Respecting XDG paths: Modrinth App uses standard data directories on Linux
        let data_home = env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".local").join("share"));

        paths.push(home.join(".sdkman").join("candidates").join("java"));
        paths.push(home.join(".minecraft").join("runtime"));
        paths.push(data_home.join("PrismLauncher").join("java"));
        paths.push(
            data_home
                .join("ModrinthApp")
                .join("meta")
                .join("java_versions"),
        );
    }

    paths
}
