use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::Path,
    sync::Arc,
};

use async_trait::async_trait;
use daedalus::minecraft::VersionInfo;
use tokio::process::Command;

use crate::{
    features::{
        events::{ProgressBarId, ProgressService, ProgressServiceExt},
        java::Java,
        minecraft::{
            utils::{get_class_paths_jar, get_lib_path},
            MinecraftDomainError, ModLoaderProcessor,
        },
        settings::LocationInfo,
    },
    processor_rules,
    shared::IoError,
    with_mut_ref,
};

pub struct ForgeProcessor<PS: ProgressService> {
    progress_service: Arc<PS>,
    location_info: Arc<LocationInfo>,
}

impl<PS: ProgressService> ForgeProcessor<PS> {
    pub fn new(progress_service: Arc<PS>, location_info: Arc<LocationInfo>) -> Self {
        Self {
            progress_service,
            location_info,
        }
    }

    async fn run_single_processor(
        processor: &daedalus::modded::Processor,
        data: &HashMap<String, daedalus::modded::SidedDataEntry>,
        libraries_dir: &Path,
        java_version: &Java,
    ) -> Result<(), MinecraftDomainError> {
        log::debug!("Running forge processor {}", processor.jar);

        let class_path: Vec<String> = with_mut_ref!(cp = processor.classpath.clone() => {
            cp.push(processor.jar.clone())
        });

        let class_path_arg =
            get_class_paths_jar(libraries_dir, &class_path, java_version.architecture())?;

        let processor_jar_path = get_lib_path(libraries_dir, &processor.jar, false)?;
        let processor_main_class = get_processor_main_class(processor_jar_path).await?.ok_or({
            MinecraftDomainError::ProcessorFailed {
                reason: format!("Could not find processor main class for {}", processor.jar),
            }
        })?;

        let processor_args = get_processor_arguments(libraries_dir, &processor.args, data)?;

        let output = Command::new(java_version.path())
            .arg("-cp")
            .arg(class_path_arg)
            .arg(processor_main_class)
            .args(processor_args)
            .output()
            .await
            .map_err(|e| IoError::with_path(e, java_version.path()))
            .map_err(|err| MinecraftDomainError::ProcessorFailed {
                reason: format!("Error running processor: {err}"),
            })?;

        if !output.status.success() {
            return Err(MinecraftDomainError::ProcessorFailed {
                reason: format!(
                    "Processor error: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }
}

#[async_trait]
impl<PS: ProgressService> ModLoaderProcessor for ForgeProcessor<PS> {
    async fn run(
        &self,
        game_version: String,
        version_jar: String,
        minecraft_dir: &Path,
        version_info: &mut VersionInfo,
        java_version: &Java,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError> {
        let Some(processors) = &version_info.processors else {
            return Ok(());
        };

        let client_path = self
            .location_info
            .version_dir(&version_jar)
            .join(format!("{version_jar}.jar"));

        let libraries_dir = self.location_info.libraries_dir();

        let Some(ref mut data) = version_info.data else {
            return Ok(());
        };

        processor_rules! {
            data;
            "SIDE":
                client => "client",
                server => "";
            "MINECRAFT_JAR":
                client => client_path.to_string_lossy(),
                server => "";
            "MINECRAFT_VERSION":
                client => game_version.clone(),
                server => "";
            "ROOT":
                client => minecraft_dir.to_string_lossy(),
                server => "";
            "LIBRARY_DIR":
                client => libraries_dir.to_string_lossy(),
                server => "";
        }

        if let Some(loading_bar) = loading_bar {
            self.progress_service
                .emit_progress_safe(loading_bar, 0.0, Some("Running forge processors"))
                .await;
        }

        let total_processors = processors.len();
        for (index, processor) in processors.iter().enumerate() {
            if let Some(sides) = &processor.sides {
                if !sides.contains(&String::from("client")) {
                    continue;
                }
            }

            Self::run_single_processor(processor, data, &libraries_dir, java_version).await?;

            if let Some(loading_bar) = loading_bar {
                let progress = 30.0 / total_processors as f64;
                let message = format!("Running forge processor {}/{}", index + 1, total_processors);
                self.progress_service
                    .emit_progress_safe(loading_bar, progress, Some(&message))
                    .await;
            }
        }

        Ok(())
    }
}

fn process_argument(
    libraries_path: &Path,
    argument: &str,
    data: &HashMap<String, daedalus::modded::SidedDataEntry>,
) -> Result<String, MinecraftDomainError> {
    // Arguments in [] are resolved to the path of a previously downloaded library
    // Check if the argument is a direct library reference [group:artifact:version]
    if let Some(stripped) = argument.strip_prefix('[') {
        if let Some(lib_key) = stripped.strip_suffix(']') {
            return get_lib_path(libraries_path, lib_key, true);
        }
    }

    let mut result = argument.to_string();

    // Arguments may contain {KEY} placeholders. We use a naive find-and-replace
    // to support composite values like "{ROOT}/libraries/".
    for (key, entry) in data {
        let placeholder = format!("{{{}}}", key);

        if result.contains(&placeholder) {
            let replacement = if let Some(inner) = entry.client.strip_prefix('[') {
                if let Some(lib_key) = inner.strip_suffix(']') {
                    // Resolve library path if the data value itself is a lib key
                    get_lib_path(libraries_path, lib_key, true)?
                } else {
                    entry.client.clone()
                }
            } else {
                entry.client.clone()
            };

            result = result.replace(&placeholder, &replacement);
        }
    }

    if result.contains('{') && result.contains('}') {
        return Err(MinecraftDomainError::ProcessorFailed {
            reason: format!("Argument contains unresolved placeholders: {}", result),
        });
    }

    Ok(result)
}
pub fn get_processor_arguments<T: AsRef<str>>(
    libraries_path: &Path,
    arguments: &[T],
    data: &HashMap<String, daedalus::modded::SidedDataEntry>,
) -> Result<Vec<String>, MinecraftDomainError> {
    arguments
        .iter()
        .map(|arg| process_argument(libraries_path, arg.as_ref(), data))
        .collect()
}

pub async fn get_processor_main_class(
    path: String,
) -> Result<Option<String>, MinecraftDomainError> {
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(&path).map_err(|e| IoError::with_path(e, &path))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|_| MinecraftDomainError::ProcessorFailed {
                reason: format!("Cannot read processor at {}", path),
            })?;

        let manifest = archive.by_name("META-INF/MANIFEST.MF").map_err(|_| {
            MinecraftDomainError::ProcessorFailed {
                reason: format!("Cannot read processor manifest at {}", path),
            }
        })?;

        let reader = BufReader::new(manifest);

        for line in reader.lines() {
            let line = line.map_err(IoError::from)?;
            let trimmed_line = line.trim();

            if let Some(class) = trimmed_line.strip_prefix("Main-Class:") {
                return Ok(Some(class.trim().to_string()));
            }
        }

        Ok(None)
    })
    .await
    .map_err(|err| MinecraftDomainError::ProcessorFailed {
        reason: err.to_string(),
    })?
}
