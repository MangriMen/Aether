use std::{path::Path, process::Stdio, time::Duration};

use tokio::process::Command;

use crate::{features::java::JavaDomainError, shared::limits::JAVA_PROBE_TIMEOUT};

#[derive(Debug, Default)]
pub struct JavaProperties {
    pub version: Option<String>,
    pub architecture: Option<String>,
}

/// Asks the runtime at `path` for its properties, giving up after [`JAVA_PROBE_TIMEOUT`].
pub async fn get_java_properties(path: &Path) -> Result<JavaProperties, JavaDomainError> {
    get_java_properties_with(path, JAVA_PROBE_TIMEOUT).await
}

async fn get_java_properties_with(
    path: &Path,
    timeout: Duration,
) -> Result<JavaProperties, JavaDomainError> {
    let mut command = Command::new(path);
    command
        .arg("-XshowSettings:properties")
        .arg("-version")
        // Options picked up from the environment would add noise to the output, or change it.
        .env_remove("_JAVA_OPTIONS")
        .env_remove("JAVA_TOOL_OPTIONS")
        .env_remove("JDK_JAVA_OPTIONS")
        .stdin(Stdio::null())
        // A probe that times out is dropped, and its process must not outlive it.
        .kill_on_drop(true);
    // No console window may flash for a probe.
    #[cfg(windows)]
    command.creation_flags(windows::Win32::System::Threading::CREATE_NO_WINDOW.0);

    let output = tokio::time::timeout(timeout, command.output())
        .await
        .map_err(|_| JavaDomainError::PropertiesTimeout {
            path: path.to_path_buf(),
            seconds: timeout.as_secs(),
        })?
        .map_err(|source| JavaDomainError::FailedToGetProperties {
            path: path.to_path_buf(),
            source: std::sync::Arc::new(source),
        })?;

    let mut combined_output = String::new();
    combined_output.push_str(&String::from_utf8_lossy(&output.stdout));
    combined_output.push_str(&String::from_utf8_lossy(&output.stderr));

    let mut version = None;
    let mut architecture = None;

    for line in combined_output.lines() {
        let (key, value) = line.split_once('=').unwrap_or(("", ""));

        match key.trim() {
            "os.arch" => architecture = Some(value.trim().to_string()),
            "java.version" => version = Some(value.trim().to_string()),
            _ => {}
        }
    }

    Ok(JavaProperties {
        version,
        architecture,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_keep_the_path_and_the_io_source_when_java_cannot_be_started() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("no-java");

        let err = get_java_properties(&missing).await.unwrap_err();

        assert!(matches!(
            err,
            JavaDomainError::FailedToGetProperties { ref path, ref source }
                if path == &missing && source.kind() == std::io::ErrorKind::NotFound
        ));
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn should_time_out_and_kill_the_process_when_java_does_not_answer() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let pid_file = tmp.path().join("pid");
        let script = tmp.path().join("java");
        // `exec` keeps the pid of the script: the one the probe kills.
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\necho $$ > '{}'\nexec sleep 60\n",
                pid_file.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

        let err = get_java_properties_with(&script, Duration::from_millis(500))
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            JavaDomainError::PropertiesTimeout { ref path, seconds: 0 } if path == &script
        ));

        let pid = std::fs::read_to_string(&pid_file)
            .unwrap()
            .trim()
            .to_string();
        let proc_dir = Path::new("/proc").join(&pid);
        // The kill is asynchronous: give the process a moment to be reaped.
        for _ in 0..50 {
            if !proc_dir.exists() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        panic!("the probe left process {pid} running");
    }
}
