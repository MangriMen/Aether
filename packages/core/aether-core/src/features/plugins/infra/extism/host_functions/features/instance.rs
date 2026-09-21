use aether_core_plugin_api::v0::{
    ContentFileDto, NewInstanceDto, ReadInstanceFileParamsDto, WriteInstanceFileParamsDto,
};
use dashmap::DashMap;
use extism::{convert::Msgpack, host_fn};
use path_slash::PathBufExt;

use crate::{
    core::app::AetherContainer,
    features::instance::{
        ChangeContentState, ContentManagementPort, ContentStateAction, GetInstanceUseCasePort,
        InstanceCrudPort, InstanceFileService, InstanceServicesPort,
    },
    shared::execute_async::infra::execute_async,
};

use super::super::{super::mappers::to_extism_res, PluginContext};

// ── Testable business logic ──

pub(crate) async fn handle_instance_get_dir(
    id: &str,
    location_info: &crate::features::settings::LocationInfo,
) -> crate::Result<String> {
    let dir = location_info.instance_dir(id);
    let relative_path = dir
        .strip_prefix(location_info.config_dir())
        .map_err(|_| crate::ErrorKind::CoreError("Strip prefix error".to_owned()))?
        .to_path_buf();

    Ok(format!("/{}", relative_path.to_slash_lossy()))
}

pub(crate) async fn handle_instance_plugin_get_dir(
    plugin_id: &str,
    instance_id: &str,
    location_info: &crate::features::settings::LocationInfo,
) -> crate::Result<String> {
    let dir = location_info.instance_plugin_dir(instance_id, plugin_id);
    let relative_path = dir
        .strip_prefix(location_info.config_dir())
        .map_err(|_| crate::ErrorKind::CoreError("Strip prefix error".to_owned()))?
        .to_path_buf();

    Ok(format!("/{}", relative_path.to_slash_lossy()))
}

pub(crate) async fn handle_instance_create(
    dto: NewInstanceDto,
    container: &AetherContainer,
) -> crate::Result<String> {
    container
        .create_instance_use_case()
        .execute(dto.into())
        .await
        .map_err(Into::into)
}

pub(crate) async fn handle_list_content(
    id: String,
    container: &AetherContainer,
) -> crate::Result<DashMap<String, ContentFileDto>> {
    container
        .list_content_use_case()
        .execute(id)
        .await
        .map_err(Into::into)
        .map(|map| {
            map.into_iter()
                .map(|(key, content)| (key, content.into()))
                .collect()
        })
}

pub(crate) async fn handle_enable_contents(
    instance_id: String,
    content_paths: Vec<String>,
    container: &AetherContainer,
) -> crate::Result<()> {
    container
        .change_content_state_use_case()
        .execute(ChangeContentState::multiple(
            instance_id,
            content_paths,
            ContentStateAction::Enable,
        ))
        .await
        .map_err(Into::into)
}

pub(crate) async fn handle_disable_contents(
    instance_id: String,
    content_paths: Vec<String>,
    container: &AetherContainer,
) -> crate::Result<()> {
    container
        .change_content_state_use_case()
        .execute(ChangeContentState::multiple(
            instance_id,
            content_paths,
            ContentStateAction::Disable,
        ))
        .await
        .map_err(Into::into)
}

/// Refuse early if the instance is not one the core knows about.
///
/// Without this a plugin could name any id at all and the file service would happily resolve a
/// path under `instances/<that id>/`, creating a directory the launcher never listed.
async fn require_instance(
    instance_id: &str,
    get_instance: &dyn GetInstanceUseCasePort,
) -> crate::Result<()> {
    get_instance
        .execute(instance_id.to_owned())
        .await
        .map(|_| ())
        .map_err(Into::into)
}

/// `write_instance_file` over the two ports it actually needs.
///
/// Separate from [`handle_write_instance_file`] so the security-relevant sequence — existence
/// check first, then the file service — can be exercised without standing up a whole
/// `AetherContainer`.
pub(crate) async fn write_instance_file_with(
    plugin_id: &str,
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

    require_instance(&params.instance_id, get_instance).await?;

    file_service
        .write_instance_file(&params.instance_id, &params.relative_path, &params.bytes)
        .await
        .map_err(|err| {
            log::warn!(target: "plugin", "[{plugin_id}]: write_instance_file failed: {err}");
            err.into()
        })
}

/// `read_instance_file` over the two ports it actually needs. See [`write_instance_file_with`].
pub(crate) async fn read_instance_file_with(
    plugin_id: &str,
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

    require_instance(&params.instance_id, get_instance).await?;

    file_service
        .read_instance_file(&params.instance_id, &params.relative_path)
        .await
        .map_err(|err| {
            log::warn!(target: "plugin", "[{plugin_id}]: read_instance_file failed: {err}");
            err.into()
        })
}

/// Handle `write_instance_file` — the only way a plugin can put a file into an instance (T-1.3).
///
/// The plugin passes an instance id and a path relative to that instance, never a WASI path:
/// `/instances/...` and `/mnt/d/...` are not accepted here, so there is no second spelling of
/// the same file to keep in sync. Everything else — the segment rules, the symlink check and
/// the size cap — lives in the file service, which is also what the rest of the core writes
/// through, so `PackStorage` does not drift away from what is on disk.
pub(crate) async fn handle_write_instance_file(
    plugin_id: &str,
    params: WriteInstanceFileParamsDto,
    container: &AetherContainer,
) -> crate::Result<()> {
    write_instance_file_with(
        plugin_id,
        params,
        &*container.get_instance_use_case(),
        &*container.instance_file_service(),
    )
    .await
}

/// Handle `read_instance_file` — read a file back out of an instance (T-1.3).
pub(crate) async fn handle_read_instance_file(
    plugin_id: &str,
    params: ReadInstanceFileParamsDto,
    container: &AetherContainer,
) -> crate::Result<Vec<u8>> {
    read_instance_file_with(
        plugin_id,
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
        execute_async(handle_instance_get_dir(&id, &location_info))
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
        execute_async(handle_instance_plugin_get_dir(&plugin_id, &instance_id, &location_info))
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
        drop(ctx);

        to_extism_res::<String>(
            execute_async(handle_instance_create(new_instance_dto.0, &container))
        )
    }
);

host_fn!(
pub list_content(user_data: PluginContext; id: String) -> MsgpackResult<DashMap<String, ContentFileDto>> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<DashMap<String, ContentFileDto>>(
        execute_async(handle_list_content(id, &container))
    )
});

host_fn!(
pub enable_contents(user_data: PluginContext; instance_id: String, content_paths: Msgpack<Vec<String>>) -> MsgpackResult<()> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<()>(
        execute_async(handle_enable_contents(instance_id, content_paths.0, &container))
    )
});

host_fn!(
pub disable_contents(user_data: PluginContext; instance_id: String, content_paths: Msgpack<Vec<String>>) -> MsgpackResult<()> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<()>(
        execute_async(handle_disable_contents(instance_id, content_paths.0, &container))
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
        execute_async(handle_write_instance_file(&id, params.0, &container))
    )
});

host_fn!(
pub read_instance_file(user_data: PluginContext; params: Msgpack<ReadInstanceFileParamsDto>) -> MsgpackResult<Vec<u8>> {
    let context = user_data.get()?;
    let ctx = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?;
    let id = ctx.id.clone();
    let container = ctx.upgrade_container().ok_or_else(|| anyhow::Error::msg("AetherContainer dropped before plugin call"))?;
    drop(ctx);

    to_extism_res::<Vec<u8>>(
        execute_async(handle_read_instance_file(&id, params.0, &container))
    )
});

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use tempfile::TempDir;

    use crate::features::instance::{
        Instance, InstanceBuilder, InstanceError, infra::FsInstanceFileService,
    };
    use crate::features::minecraft::ModLoader;
    use crate::features::settings::LocationInfo;

    use super::*;

    const PLUGIN_ID: &str = "test-plugin";
    const INSTANCE_ID: &str = "inst-1";

    /// Stands in for `GetInstanceUseCase`: knows about exactly one instance.
    struct OneInstance;

    #[async_trait]
    impl GetInstanceUseCasePort for OneInstance {
        async fn execute(&self, instance_id: String) -> Result<Instance, InstanceError> {
            if instance_id == INSTANCE_ID {
                Ok(InstanceBuilder::new(
                    INSTANCE_ID.to_owned(),
                    "Test".to_owned(),
                    "1.20.1".to_owned(),
                    ModLoader::Vanilla,
                )
                .build())
            } else {
                Err(InstanceError::NotFound { instance_id })
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
        let service = FsInstanceFileService::new(location_info.clone());
        service
            .create_instance_dir(INSTANCE_ID)
            .await
            .expect("create_instance_dir should succeed");
        (temp_dir, location_info, service)
    }

    fn write_params(instance_id: &str, relative_path: &str, bytes: &[u8]) -> WriteInstanceFileParamsDto {
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
    async fn should_write_the_file_inside_the_instance_directory() {
        let (_temp_dir, location_info, service) = setup().await;

        write_instance_file_with(
            PLUGIN_ID,
            write_params(INSTANCE_ID, "mods/example.jar", b"jar bytes"),
            &OneInstance,
            &service,
        )
        .await
        .expect("write_instance_file should succeed");

        let written = location_info
            .instance_dir(INSTANCE_ID)
            .join("mods")
            .join("example.jar");
        assert!(
            written.exists(),
            "the file must appear inside the instance directory"
        );
        assert_eq!(tokio::fs::read(&written).await.unwrap(), b"jar bytes");
    }

    #[tokio::test]
    async fn should_read_back_what_was_written() {
        let (_temp_dir, _location_info, service) = setup().await;

        write_instance_file_with(
            PLUGIN_ID,
            write_params(INSTANCE_ID, "pack.toml", b"name = \"test\""),
            &OneInstance,
            &service,
        )
        .await
        .unwrap();

        let bytes = read_instance_file_with(
            PLUGIN_ID,
            read_params(INSTANCE_ID, "pack.toml"),
            &OneInstance,
            &service,
        )
        .await
        .expect("read_instance_file should succeed");

        assert_eq!(bytes, b"name = \"test\"");
    }

    #[tokio::test]
    async fn should_refuse_an_unknown_instance_id() {
        let (temp_dir, _location_info, service) = setup().await;

        let err = write_instance_file_with(
            PLUGIN_ID,
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
    async fn should_refuse_a_traversal_path() {
        let (temp_dir, _location_info, service) = setup().await;

        for path in ["../escaped.txt", "mods/../../escaped.txt"] {
            write_instance_file_with(
                PLUGIN_ID,
                write_params(INSTANCE_ID, path, b"pwned"),
                &OneInstance,
                &service,
            )
            .await
            .expect_err("a traversal path must be refused");
        }

        assert!(
            !temp_dir.path().join("instances").join("escaped.txt").exists(),
            "nothing may be written outside the instance"
        );
    }

    #[tokio::test]
    async fn should_refuse_a_wasi_path_instead_of_resolving_it() {
        let (_temp_dir, _location_info, service) = setup().await;

        // The trap from the task card: these host functions take an id plus a relative path,
        // so the WASI spellings a plugin uses elsewhere must not work here.
        for path in [
            "/instances/inst-1/pack.toml",
            "/mnt/d/instances/inst-1/pack.toml",
        ] {
            write_instance_file_with(
                PLUGIN_ID,
                write_params(INSTANCE_ID, path, b"pwned"),
                &OneInstance,
                &service,
            )
            .await
            .expect_err("a WASI path must be refused");
        }
    }

    #[tokio::test]
    async fn should_refuse_reading_from_an_unknown_instance_id() {
        let (_temp_dir, _location_info, service) = setup().await;

        read_instance_file_with(
            PLUGIN_ID,
            read_params("no-such-instance", "pack.toml"),
            &OneInstance,
            &service,
        )
        .await
        .expect_err("an unknown instance id must be refused");
    }
}
