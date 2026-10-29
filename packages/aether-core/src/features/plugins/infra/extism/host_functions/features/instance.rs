use aether_core_plugin_api::v0::{
    ContentFileDto, InstanceFileBytesDto, NewInstanceDto, ReadInstanceFileParamsDto,
    WriteInstanceFileParamsDto,
};
use dashmap::DashMap;
use extism::{convert::Msgpack, host_fn};
use path_slash::PathBufExt;

use crate::{
    core::app::AetherContainer,
    features::instance::{
        ChangeContentState, ChangeContentStateUseCasePort, ContentManagementPort,
        ContentStateAction, GetInstanceUseCasePort, InstanceCrudPort, InstanceError,
        InstanceFileService, InstanceId, InstanceServicesPort, ListContentUseCasePort, NewInstance,
        infra::{check_plugin_write_allowed, check_size},
    },
    features::plugins::LoadConfig,
    features::plugins::infra::{
        pack_instances::{is_deploying_packs, record_pack_instance},
        plugin_utils::{MountSet, resolve_plugin_path},
    },
    features::settings::LocationInfo,
};

use super::super::{super::mappers::to_extism_res, PluginContext, block_on};

// ── Testable business logic ──

pub(crate) async fn handle_instance_get_dir(
    id: &str,
    location_info: &LocationInfo,
) -> crate::Result<String> {
    let id = InstanceId::try_from(id)?;
    let dir = location_info.instance_dir(&id);
    let relative_path = dir
        .strip_prefix(location_info.config_dir())
        .map_err(|_| crate::ErrorKind::CoreError("Strip prefix error".to_owned()))?
        .to_path_buf();

    Ok(format!("/{}", relative_path.to_slash_lossy()))
}

pub(crate) async fn handle_instance_plugin_get_dir(
    plugin_id: &str,
    instance_id: &str,
    location_info: &LocationInfo,
    get_instance: &dyn GetInstanceUseCasePort,
) -> crate::Result<String> {
    let instance_id = require_instance(instance_id, get_instance).await?;

    let dir = location_info.instance_plugin_dir(&instance_id, plugin_id);
    let relative_path = dir
        .strip_prefix(location_info.config_dir())
        .map_err(|_| crate::ErrorKind::CoreError("Strip prefix error".to_owned()))?
        .to_path_buf();

    Ok(format!("/{}", relative_path.to_slash_lossy()))
}

/// `for_pack` — the plugin deploys a pack into the instance: it stays `PackInstalling` until the
/// plugin's call is over (`infra::pack_instances`).
pub(crate) async fn handle_instance_create(
    plugin_id: &str,
    dto: NewInstanceDto,
    container: &AetherContainer,
    for_pack: bool,
) -> crate::Result<InstanceId> {
    let create = container.create_instance_use_case();
    let new_instance = new_instance_from_plugin(plugin_id, dto, &container.location_info())?;

    if for_pack {
        create.execute_for_pack(new_instance, None).await
    } else {
        create.execute(new_instance, None).await
    }
    .map_err(Into::into)
}

/// The icon of an instance a plugin makes is a file of the plugin's own `/cache`: a host path
/// would have the core copy any file of the user's into the asset store, where the UI reads it.
pub(crate) fn new_instance_from_plugin(
    plugin_id: &str,
    dto: NewInstanceDto,
    location_info: &LocationInfo,
) -> crate::Result<NewInstance> {
    let icon_path = dto
        .icon_path
        .as_deref()
        .map(|icon| resolve_plugin_path(location_info, plugin_id, icon, MountSet::CACHE))
        .transpose()?
        .map(|host| host.to_string_lossy().into_owned());

    let mut new_instance =
        NewInstance::try_from(dto).map_err(|err| InstanceError::ContentProviderError {
            reason: err.to_string(),
        })?;
    new_instance.icon_path = icon_path;
    Ok(new_instance)
}

pub(crate) async fn handle_list_content(
    id: String,
    get_instance: &dyn GetInstanceUseCasePort,
    list_content: &dyn ListContentUseCasePort,
) -> crate::Result<DashMap<String, ContentFileDto>> {
    let id = require_instance(&id, get_instance).await?;

    list_content
        .execute(id)
        .await
        .map_err(Into::into)
        .map(|map| {
            map.into_iter()
                .map(|(key, content)| (key, content.file.into()))
                .collect()
        })
}

/// Enable or disable content of an instance: renames files in it, so the id is checked first.
pub(crate) async fn handle_change_contents(
    instance_id: String,
    content_paths: Vec<String>,
    action: ContentStateAction,
    get_instance: &dyn GetInstanceUseCasePort,
    change_state: &dyn ChangeContentStateUseCasePort,
) -> crate::Result<()> {
    let instance_id = require_instance(&instance_id, get_instance).await?;

    change_state
        .execute(ChangeContentState::multiple(
            instance_id,
            content_paths,
            action,
        ))
        .await
        .map_err(Into::into)
}

/// Refuse an instance the core does not know: otherwise a plugin could name any id and have
/// the file service create a directory under `instances/` the launcher never listed.
///
/// The id is checked by its spelling before the lookup: an id such as `../plugins/<id>` would
/// otherwise reach outside `instances/`, to files the plugin may have planted there. Returns the
/// parsed id.
async fn require_instance(
    instance_id: &str,
    get_instance: &dyn GetInstanceUseCasePort,
) -> crate::Result<InstanceId> {
    let instance_id = InstanceId::try_from(instance_id)?;

    get_instance.execute(instance_id.clone()).await?;

    Ok(instance_id)
}

/// `write_instance_file` over the two ports it needs, so its checks can be tested without a
/// whole `AetherContainer`.
///
/// `max_file_bytes` is the most this plugin's memory holds of a file
/// ([`PluginSandboxLimits::max_instance_file_bytes_within`]): a payload over it is refused as too
/// large rather than left to run the plugin out of memory.
pub(crate) async fn write_instance_file_with(
    plugin_id: &str,
    max_file_bytes: u64,
    params: WriteInstanceFileParamsDto,
    get_instance: &dyn GetInstanceUseCasePort,
    file_service: &dyn InstanceFileService,
) -> crate::Result<()> {
    log::debug!(
        target: "plugin",
        "[{plugin_id}]: write_instance_file {}/{} ({} bytes)",
        params.instance_id,
        params.relative_path,
        params.bytes.len()
    );

    // `.metadata` is the core's, apart from this plugin's own subdirectory in it.
    check_plugin_write_allowed(plugin_id, &params.relative_path)?;
    check_size(
        &params.relative_path,
        params.bytes.len() as u64,
        max_file_bytes,
    )?;

    let instance_id = require_instance(&params.instance_id, get_instance).await?;

    file_service
        .write_instance_file(&instance_id, &params.relative_path, &params.bytes)
        .await
        .map_err(|err| {
            log::warn!(target: "plugin", "[{plugin_id}]: write_instance_file failed: {err}");
            err.into()
        })
}

/// `read_instance_file` over the two ports it actually needs. See [`write_instance_file_with`].
pub(crate) async fn read_instance_file_with(
    plugin_id: &str,
    max_file_bytes: u64,
    params: ReadInstanceFileParamsDto,
    get_instance: &dyn GetInstanceUseCasePort,
    file_service: &dyn InstanceFileService,
) -> crate::Result<Vec<u8>> {
    log::debug!(
        target: "plugin",
        "[{plugin_id}]: read_instance_file {}/{}",
        params.instance_id,
        params.relative_path
    );

    let instance_id = require_instance(&params.instance_id, get_instance).await?;

    let bytes = file_service
        .read_instance_file(&instance_id, &params.relative_path)
        .await
        .map_err(|err| {
            log::warn!(target: "plugin", "[{plugin_id}]: read_instance_file failed: {err}");
            crate::Error::from(err)
        })?;

    // The file service bounds a file by the sandbox cap; a plugin with less memory than that
    // would still be run out of it by the copies of the bytes the call makes.
    check_size(&params.relative_path, bytes.len() as u64, max_file_bytes)?;

    Ok(bytes)
}

/// What a file passing through this plugin's memory may weigh at most: its manifest's
/// `memoryLimit`, or the default one, shared by the copies the call makes.
fn max_file_bytes_of(plugin_id: &str, container: &AetherContainer) -> crate::Result<u64> {
    let manifest = container
        .plugins()
        .registry
        .get_manifest(plugin_id)
        .map_err(|err| crate::ErrorKind::from(err).as_error())?;
    let LoadConfig::Extism { memory_limit, .. } = &manifest.load;

    Ok(container
        .sandbox_limits()
        .max_instance_file_bytes_within(*memory_limit))
}

/// Handle `write_instance_file` — the only way a plugin can put a file into an instance.
///
/// Takes an instance id and a relative path, never a WASI path, so a file has one spelling.
/// The path rules, symlink check and size cap live in the file service the rest of the core
/// writes through too.
pub(crate) async fn handle_write_instance_file(
    plugin_id: &str,
    params: WriteInstanceFileParamsDto,
    container: &AetherContainer,
) -> crate::Result<()> {
    write_instance_file_with(
        plugin_id,
        max_file_bytes_of(plugin_id, container)?,
        params,
        &*container.get_instance_use_case(),
        &*container.instance_file_service(),
    )
    .await
}

/// Handle `read_instance_file` — read a file back out of an instance.
pub(crate) async fn handle_read_instance_file(
    plugin_id: &str,
    params: ReadInstanceFileParamsDto,
    container: &AetherContainer,
) -> crate::Result<Vec<u8>> {
    read_instance_file_with(
        plugin_id,
        max_file_bytes_of(plugin_id, container)?,
        params,
        &*container.get_instance_use_case(),
        &*container.instance_file_service(),
    )
    .await
}

// ── Extism host function wrappers ──

host_fn!(
pub instance_get_dir(user_data: PluginContext; id: String) -> MsgpackResult<String> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    let location_info = container.location_info();
    drop(ctx);

    to_extism_res::<String>(
        block_on(handle_instance_get_dir(&id, &location_info))
    )
});

host_fn!(
pub instance_plugin_get_dir(user_data: PluginContext; instance_id: String) -> MsgpackResult<String> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let plugin_id = ctx.id.clone();
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    let location_info = container.location_info();
    drop(ctx);

    to_extism_res::<String>(
        block_on(handle_instance_plugin_get_dir(
            &plugin_id,
            &instance_id,
            &location_info,
            &*container.get_instance_use_case(),
        ))
    )
});

host_fn!(
    pub instance_create(
        user_data: PluginContext;
        new_instance_dto: Msgpack<NewInstanceDto>
    ) -> MsgpackResult<String> {
        let context = user_data.get()?;
        let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
        let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
        let plugin_id = ctx.id.clone();
        drop(ctx);

        // Checked and recorded on this thread, the one of the plugin's call: `block_on` drives
        // the creation here too.
        let for_pack = new_instance_dto.0.pack_info.is_some() && is_deploying_packs(&plugin_id);
        let result = block_on(handle_instance_create(&plugin_id, new_instance_dto.0, &container, for_pack));
        if for_pack && let Ok(instance_id) = &result {
            record_pack_instance(&plugin_id, instance_id);
        }

        to_extism_res::<String>(result.map(String::from))
    }
);

host_fn!(
pub list_content(user_data: PluginContext; id: String) -> MsgpackResult<DashMap<String, ContentFileDto>> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<DashMap<String, ContentFileDto>>(
        block_on(handle_list_content(
            id,
            &*container.get_instance_use_case(),
            &*container.list_content_use_case(),
        ))
    )
});

host_fn!(
pub enable_contents(user_data: PluginContext; instance_id: String, content_paths: Msgpack<Vec<String>>) -> MsgpackResult<()> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<()>(
        block_on(handle_change_contents(
            instance_id,
            content_paths.0,
            ContentStateAction::Enable,
            &*container.get_instance_use_case(),
            &*container.change_content_state_use_case(),
        ))
    )
});

host_fn!(
pub disable_contents(user_data: PluginContext; instance_id: String, content_paths: Msgpack<Vec<String>>) -> MsgpackResult<()> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<()>(
        block_on(handle_change_contents(
            instance_id,
            content_paths.0,
            ContentStateAction::Disable,
            &*container.get_instance_use_case(),
            &*container.change_content_state_use_case(),
        ))
    )
});

host_fn!(
pub write_instance_file(user_data: PluginContext; params: Msgpack<WriteInstanceFileParamsDto>) -> MsgpackResult<()> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.clone();
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<()>(
        block_on(handle_write_instance_file(&id, params.0, &container))
    )
});

host_fn!(
pub read_instance_file(user_data: PluginContext; params: Msgpack<ReadInstanceFileParamsDto>) -> MsgpackResult<InstanceFileBytesDto> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.clone();
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<InstanceFileBytesDto>(
        block_on(handle_read_instance_file(&id, params.0, &container)).map(InstanceFileBytesDto)
    )
});

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use aether_core_plugin_api::v0::ModLoaderDto;
    use async_trait::async_trait;
    use tempfile::TempDir;

    use crate::features::instance::{
        ContentFileView, Instance, InstanceBuilder, InstanceError, infra::FsInstanceFileService,
    };
    use crate::features::minecraft::ModLoader;
    use crate::features::settings::LocationInfo;

    use super::*;

    const PLUGIN_ID: &str = "test-plugin";
    const INSTANCE_ID: &str = "inst-1";
    const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;

    /// Stands in for `GetInstanceUseCase`: knows about exactly one instance.
    struct OneInstance;

    #[async_trait]
    impl GetInstanceUseCasePort for OneInstance {
        async fn execute(&self, instance_id: InstanceId) -> Result<Instance, InstanceError> {
            if instance_id.as_str() == INSTANCE_ID {
                Ok(InstanceBuilder::new(
                    InstanceId::try_from(INSTANCE_ID).unwrap(),
                    "Test".to_owned(),
                    "1.20.1".to_owned(),
                    ModLoader::Vanilla,
                )
                .build())
            } else {
                Err(InstanceError::NotFound {
                    instance_id: instance_id.into(),
                })
            }
        }
    }

    /// The real file service over a temp directory, with the instance directory already made —
    /// the same arrangement the host function sees in production.
    async fn setup() -> (TempDir, Arc<LocationInfo>, FsInstanceFileService) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let location_info = Arc::new(LocationInfo::new(
            temp_dir.path().join("settings"),
            temp_dir.path().to_path_buf(),
        ));
        let service = FsInstanceFileService::new(
            location_info.clone(),
            crate::shared::limits::PluginSandboxLimits::default().max_instance_file_bytes,
        );
        service
            .create_instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
            .await
            .expect("create_instance_dir should succeed");
        (temp_dir, location_info, service)
    }

    fn write_params(
        instance_id: &str,
        relative_path: &str,
        bytes: &[u8],
    ) -> WriteInstanceFileParamsDto {
        WriteInstanceFileParamsDto {
            instance_id: instance_id.to_owned(),
            relative_path: relative_path.to_owned(),
            bytes: bytes.to_vec(),
        }
    }

    fn read_params(instance_id: &str, relative_path: &str) -> ReadInstanceFileParamsDto {
        ReadInstanceFileParamsDto {
            instance_id: instance_id.to_owned(),
            relative_path: relative_path.to_owned(),
        }
    }

    #[tokio::test]
    async fn should_write_the_file_inside_the_instance_directory_when_the_id_is_known() {
        let (_temp_dir, location_info, service) = setup().await;

        write_instance_file_with(
            PLUGIN_ID,
            MAX_FILE_BYTES,
            write_params(INSTANCE_ID, "mods/example.jar", b"jar bytes"),
            &OneInstance,
            &service,
        )
        .await
        .expect("write_instance_file should succeed");

        let written = location_info
            .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
            .join("mods")
            .join("example.jar");
        assert!(
            written.exists(),
            "the file must appear inside the instance directory"
        );
        assert_eq!(tokio::fs::read(&written).await.unwrap(), b"jar bytes");
    }

    #[tokio::test]
    async fn should_read_back_what_was_written_when_the_file_exists() {
        let (_temp_dir, _location_info, service) = setup().await;

        write_instance_file_with(
            PLUGIN_ID,
            MAX_FILE_BYTES,
            write_params(INSTANCE_ID, "pack.toml", b"name = \"test\""),
            &OneInstance,
            &service,
        )
        .await
        .unwrap();

        let bytes = read_instance_file_with(
            PLUGIN_ID,
            MAX_FILE_BYTES,
            read_params(INSTANCE_ID, "pack.toml"),
            &OneInstance,
            &service,
        )
        .await
        .expect("read_instance_file should succeed");

        assert_eq!(bytes, b"name = \"test\"");
    }

    #[tokio::test]
    async fn should_refuse_a_write_when_the_instance_id_is_unknown() {
        let (temp_dir, _location_info, service) = setup().await;

        let err = write_instance_file_with(
            PLUGIN_ID,
            MAX_FILE_BYTES,
            write_params("no-such-instance", "pack.toml", b"pwned"),
            &OneInstance,
            &service,
        )
        .await
        .expect_err("an unknown instance id must be refused");
        assert!(err.to_string().contains("not found"), "{err}");

        assert!(
            !temp_dir
                .path()
                .join("instances")
                .join("no-such-instance")
                .exists(),
            "a refused write must not have created the directory"
        );
    }

    #[tokio::test]
    async fn should_refuse_a_write_over_the_memory_of_the_plugin_when_it_is_under_the_sandbox_cap()
    {
        let (_temp_dir, location_info, service) = setup().await;

        let err = write_instance_file_with(
            PLUGIN_ID,
            4,
            write_params(INSTANCE_ID, "big.bin", b"12345"),
            &OneInstance,
            &service,
        )
        .await
        .expect_err("a payload over the plugin's cap must be refused");

        assert!(
            matches!(
                err.raw.as_ref(),
                crate::ErrorKind::InstanceError(InstanceError::FileTooLarge {
                    size: 5,
                    limit: 4,
                    ..
                })
            ),
            "{err}"
        );
        assert!(
            !location_info
                .instance_dir(&InstanceId::try_from(INSTANCE_ID).unwrap())
                .join("big.bin")
                .exists()
        );
    }

    #[tokio::test]
    async fn should_refuse_a_read_over_the_memory_of_the_plugin_when_it_is_under_the_sandbox_cap() {
        let (_temp_dir, _location_info, service) = setup().await;
        write_instance_file_with(
            PLUGIN_ID,
            MAX_FILE_BYTES,
            write_params(INSTANCE_ID, "big.bin", b"12345"),
            &OneInstance,
            &service,
        )
        .await
        .unwrap();

        let err = read_instance_file_with(
            PLUGIN_ID,
            4,
            read_params(INSTANCE_ID, "big.bin"),
            &OneInstance,
            &service,
        )
        .await
        .expect_err("a file over the plugin's cap must not be handed to it");

        assert!(
            matches!(
                err.raw.as_ref(),
                crate::ErrorKind::InstanceError(InstanceError::FileTooLarge {
                    size: 5,
                    limit: 4,
                    ..
                })
            ),
            "{err}"
        );
    }

    #[tokio::test]
    async fn should_refuse_a_write_when_the_path_traverses_out_of_the_instance() {
        let (temp_dir, _location_info, service) = setup().await;

        for path in ["../escaped.txt", "mods/../../escaped.txt"] {
            write_instance_file_with(
                PLUGIN_ID,
                MAX_FILE_BYTES,
                write_params(INSTANCE_ID, path, b"pwned"),
                &OneInstance,
                &service,
            )
            .await
            .expect_err("a traversal path must be refused");
        }

        assert!(
            !temp_dir
                .path()
                .join("instances")
                .join("escaped.txt")
                .exists(),
            "nothing may be written outside the instance"
        );
    }

    #[tokio::test]
    async fn should_refuse_a_wasi_path_when_writing_instead_of_resolving_it() {
        let (temp_dir, _location_info, service) = setup().await;

        // These host functions take an id plus a relative path, so the WASI spellings a plugin
        // uses elsewhere must not work here.
        for path in [
            "/instances/inst-1/pack.toml",
            "/mnt/d/instances/inst-1/pack.toml",
        ] {
            write_instance_file_with(
                PLUGIN_ID,
                MAX_FILE_BYTES,
                write_params(INSTANCE_ID, path, b"pwned"),
                &OneInstance,
                &service,
            )
            .await
            .expect_err("a WASI path must be refused");
        }

        let instance_dir = temp_dir.path().join("instances").join(INSTANCE_ID);
        assert!(
            !instance_dir.join("pack.toml").exists()
                && !instance_dir.join("instances").exists()
                && !instance_dir.join("mnt").exists(),
            "a refused WASI-path write must not create the file"
        );
    }

    #[tokio::test]
    async fn should_refuse_a_read_when_the_instance_id_is_unknown() {
        let (_temp_dir, _location_info, service) = setup().await;

        let err = read_instance_file_with(
            PLUGIN_ID,
            MAX_FILE_BYTES,
            read_params("no-such-instance", "pack.toml"),
            &OneInstance,
            &service,
        )
        .await
        .expect_err("an unknown instance id must be refused");
        assert!(err.to_string().contains("not found"), "{err}");
    }

    #[tokio::test]
    async fn should_refuse_a_plugin_write_when_the_path_is_in_instance_metadata() {
        let (_temp_dir, location_info, service) = setup().await;

        // The core's own metadata file: rewriting it would let a plugin restate the install
        // stage or the loader of an instance.
        let metadata_file =
            location_info.instance_metadata_file(&InstanceId::try_from(INSTANCE_ID).unwrap());
        tokio::fs::create_dir_all(metadata_file.parent().unwrap())
            .await
            .unwrap();
        tokio::fs::write(&metadata_file, b"real metadata")
            .await
            .unwrap();

        write_instance_file_with(
            PLUGIN_ID,
            MAX_FILE_BYTES,
            write_params(INSTANCE_ID, ".metadata/instance.json", b"pwned"),
            &OneInstance,
            &service,
        )
        .await
        .expect_err("a plugin must not write into `.metadata`");

        assert_eq!(
            tokio::fs::read(&metadata_file).await.unwrap(),
            b"real metadata",
            "the refused write must not have touched the file"
        );
    }

    #[tokio::test]
    async fn should_allow_a_plugin_write_when_the_path_is_in_its_own_metadata_subdirectory() {
        let (_temp_dir, location_info, service) = setup().await;

        // Where the core itself puts a plugin's per-instance state, and where packwiz keeps
        // `packwiz.toml`.
        let relative_path = format!(".metadata/plugins/{PLUGIN_ID}/packwiz.toml");

        write_instance_file_with(
            PLUGIN_ID,
            MAX_FILE_BYTES,
            write_params(INSTANCE_ID, &relative_path, b"pack_path = \"x\""),
            &OneInstance,
            &service,
        )
        .await
        .expect("a plugin owns its own subdirectory of `.metadata`");

        let written = location_info
            .instance_plugin_dir(&InstanceId::try_from(INSTANCE_ID).unwrap(), PLUGIN_ID)
            .join("packwiz.toml");
        assert!(
            written.exists(),
            "the file must land in the plugin's own directory"
        );
    }

    #[tokio::test]
    async fn should_refuse_a_plugin_write_when_the_path_is_in_another_plugins_directory() {
        let (_temp_dir, location_info, service) = setup().await;

        write_instance_file_with(
            PLUGIN_ID,
            MAX_FILE_BYTES,
            write_params(
                INSTANCE_ID,
                ".metadata/plugins/other-plugin/settings.toml",
                b"pwned",
            ),
            &OneInstance,
            &service,
        )
        .await
        .expect_err("one plugin must not write into another's directory");

        let other = location_info
            .instance_plugin_dir(&InstanceId::try_from(INSTANCE_ID).unwrap(), "other-plugin")
            .join("settings.toml");
        assert!(
            !other.exists(),
            "the refused write must not land in the other plugin's directory"
        );
    }

    #[tokio::test]
    async fn should_allow_a_read_when_the_path_is_in_instance_metadata() {
        let (_temp_dir, location_info, service) = setup().await;

        // Only writing is narrowed: the instances directory is mounted into the sandbox and
        // stays readable, so refusing the read here would buy nothing.
        let metadata_file =
            location_info.instance_metadata_file(&InstanceId::try_from(INSTANCE_ID).unwrap());
        tokio::fs::create_dir_all(metadata_file.parent().unwrap())
            .await
            .unwrap();
        tokio::fs::write(&metadata_file, b"real metadata")
            .await
            .unwrap();

        let bytes = read_instance_file_with(
            PLUGIN_ID,
            MAX_FILE_BYTES,
            read_params(INSTANCE_ID, ".metadata/instance.json"),
            &OneInstance,
            &service,
        )
        .await
        .expect("reading is not narrowed");

        assert_eq!(bytes, b"real metadata");
    }

    /// Stands in for a storage that would find an instance under any id, as one planted outside
    /// `instances/` would be found: only the id check stands between it and the call.
    struct AnyInstance;

    #[async_trait]
    impl GetInstanceUseCasePort for AnyInstance {
        async fn execute(&self, _instance_id: InstanceId) -> Result<Instance, InstanceError> {
            Ok(InstanceBuilder::new(
                InstanceId::generate(),
                "Test".to_owned(),
                "1.20.1".to_owned(),
                ModLoader::Vanilla,
            )
            .build())
        }
    }

    #[derive(Default)]
    struct RecordingContent {
        called: AtomicBool,
    }

    #[async_trait]
    impl ListContentUseCasePort for RecordingContent {
        async fn execute(
            &self,
            _instance_id: InstanceId,
        ) -> Result<DashMap<String, ContentFileView>, InstanceError> {
            self.called.store(true, Ordering::SeqCst);
            Ok(DashMap::new())
        }
    }

    #[async_trait]
    impl ChangeContentStateUseCasePort for RecordingContent {
        async fn execute(&self, _input: ChangeContentState) -> Result<(), InstanceError> {
            self.called.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    /// `../plugins/x` and an absolute path both lead out of `instances/`.
    fn escaping_ids(temp_dir: &TempDir) -> Vec<String> {
        vec![
            "../plugins/x".to_owned(),
            "..\\plugins\\x".to_owned(),
            temp_dir
                .path()
                .join("plugins")
                .join("x")
                .to_string_lossy()
                .into_owned(),
        ]
    }

    #[tokio::test]
    async fn should_refuse_to_change_contents_when_the_instance_id_leaves_instances() {
        let (temp_dir, _location_info, _service) = setup().await;

        for id in escaping_ids(&temp_dir) {
            for action in [ContentStateAction::Enable, ContentStateAction::Disable] {
                let change = RecordingContent::default();
                handle_change_contents(
                    id.clone(),
                    vec!["verification.json".to_owned()],
                    action,
                    &AnyInstance,
                    &change,
                )
                .await
                .expect_err("an id outside `instances/` must be refused");
                assert!(!change.called.load(Ordering::SeqCst), "{id}");
            }
        }
    }

    #[tokio::test]
    async fn should_refuse_to_change_contents_when_the_instance_is_unknown() {
        let change = RecordingContent::default();

        handle_change_contents(
            "no-such-instance".to_owned(),
            vec!["mods/a.jar".to_owned()],
            ContentStateAction::Disable,
            &OneInstance,
            &change,
        )
        .await
        .expect_err("an unknown instance must be refused");
        assert!(!change.called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn should_change_contents_when_the_instance_is_known() {
        let change = RecordingContent::default();

        handle_change_contents(
            INSTANCE_ID.to_owned(),
            vec!["mods/a.jar".to_owned()],
            ContentStateAction::Disable,
            &OneInstance,
            &change,
        )
        .await
        .expect("a known instance is accepted");
        assert!(change.called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn should_refuse_to_list_content_when_the_instance_id_leaves_instances() {
        let (temp_dir, _location_info, _service) = setup().await;

        for id in escaping_ids(&temp_dir) {
            let list = RecordingContent::default();
            handle_list_content(id.clone(), &AnyInstance, &list)
                .await
                .expect_err("an id outside `instances/` must be refused");
            assert!(!list.called.load(Ordering::SeqCst), "{id}");
        }

        let list = RecordingContent::default();
        handle_list_content("no-such-instance".to_owned(), &OneInstance, &list)
            .await
            .expect_err("an unknown instance must be refused");
        assert!(!list.called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn should_refuse_the_plugin_dir_when_the_instance_id_is_not_a_known_instance() {
        let (temp_dir, location_info, _service) = setup().await;

        for id in escaping_ids(&temp_dir) {
            handle_instance_plugin_get_dir(PLUGIN_ID, &id, &location_info, &AnyInstance)
                .await
                .expect_err("an id outside `instances/` must be refused");
        }

        handle_instance_plugin_get_dir(PLUGIN_ID, "no-such-instance", &location_info, &OneInstance)
            .await
            .expect_err("an unknown instance must be refused");

        let dir =
            handle_instance_plugin_get_dir(PLUGIN_ID, INSTANCE_ID, &location_info, &OneInstance)
                .await
                .expect("a known instance is accepted");
        assert_eq!(
            dir,
            format!("/instances/{INSTANCE_ID}/.metadata/plugins/{PLUGIN_ID}")
        );
    }

    #[tokio::test]
    async fn should_refuse_the_instance_dir_when_the_instance_id_leaves_instances() {
        let (temp_dir, location_info, _service) = setup().await;

        for id in escaping_ids(&temp_dir) {
            handle_instance_get_dir(&id, &location_info)
                .await
                .expect_err("an id outside `instances/` must be refused");
        }

        assert_eq!(
            handle_instance_get_dir(INSTANCE_ID, &location_info)
                .await
                .unwrap(),
            format!("/instances/{INSTANCE_ID}")
        );
    }

    fn dto_with_icon(icon_path: Option<&str>) -> NewInstanceDto {
        NewInstanceDto {
            name: "Pack".to_owned(),
            game_version: "1.21".to_owned(),
            mod_loader: ModLoaderDto::Vanilla,
            loader_version: None,
            icon_path: icon_path.map(str::to_owned),
            skip_install_instance: None,
            pack_info: None,
        }
    }

    #[tokio::test]
    async fn should_translate_the_icon_to_a_host_path_when_it_is_in_the_plugin_cache() {
        let (_temp_dir, location_info, _service) = setup().await;
        let cache = location_info.plugin_cache_dir(PLUGIN_ID);
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::write(cache.join("icon.png"), b"png").unwrap();

        let new_instance = new_instance_from_plugin(
            PLUGIN_ID,
            dto_with_icon(Some("/cache/icon.png")),
            &location_info,
        )
        .expect("an icon in the plugin cache is accepted");

        let icon = new_instance.icon_path.expect("the icon is kept");
        assert!(
            std::path::Path::new(&icon).ends_with("icon.png")
                && std::path::Path::new(&icon).is_absolute(),
            "{icon}"
        );
    }

    #[tokio::test]
    async fn should_refuse_the_icon_when_it_is_a_host_path() {
        let (temp_dir, location_info, _service) = setup().await;
        let secret = temp_dir.path().join("secret.png");
        std::fs::write(&secret, b"png").unwrap();

        for icon in [
            secret.to_string_lossy().into_owned(),
            "/cache/../../secret.png".to_owned(),
        ] {
            new_instance_from_plugin(PLUGIN_ID, dto_with_icon(Some(&icon)), &location_info)
                .expect_err("an icon outside the plugin cache must be refused");
        }
    }

    #[tokio::test]
    async fn should_keep_no_icon_when_the_plugin_names_none() {
        let (_temp_dir, location_info, _service) = setup().await;

        let new_instance =
            new_instance_from_plugin(PLUGIN_ID, dto_with_icon(None), &location_info).unwrap();

        assert_eq!(new_instance.icon_path, None);
    }
}
