use std::{collections::HashMap, sync::Arc};

use aether_core_plugin_api::v0::{
    ContentSearchParamsDto, ContentSearchResultDto, ContentVersionDto, InstallPlanDto,
    OverrideLocationDto, OverrideSourceDto, PackMetadataDto, PackSourceDto,
    PluginCheckCompatibilityParamsDto, ResolveFileParamsDto, ResolveInstallPlanParamsDto,
    ResolvedFileDto,
};
use async_trait::async_trait;
use extism_convert::Msgpack;
use serde::{Serialize, de::DeserializeOwned};

use crate::features::{
    instance::{
        ContentCompatibilityCheckParams, ContentCompatibilityResult, ContentItem, ContentProvider,
        ContentProviderCapabilityMetadata, ContentSearchParams, ContentSearchResult,
        ContentVersion, InstallFileEntry, InstallPlan, InstallTarget, Instance, InstanceError,
        OverrideLocation, OverrideSource, PackMetadata, PackSource, ProviderId,
        ResolveInstallPlanParams, ResolveTarget, ResolvedFile,
    },
    plugins::{
        PluginCheckCompatibilityParams, PluginContentProviderCapability, PluginError, PluginHandle,
    },
    settings::LocationInfo,
};

use crate::features::plugins::infra::extism::models::PluginInstanceExt;
use crate::features::plugins::infra::pack_instances::deploying_packs;
use crate::features::plugins::infra::plugin_utils::{
    MountSet, host_path_to_plugin, resolve_plugin_path,
};

/// A content provider implemented by a plugin, calling the functions its `capabilities.json`
/// names.
///
/// The plugin boundary for what a provider returns: hashes are validated, a plan has to answer
/// the request, and every plugin-side path is translated or refused, so nothing past the proxy
/// sees one.
pub struct PluginContentProviderProxy {
    plugin_id: String,
    instance: PluginHandle,
    capability: PluginContentProviderCapability,
    location_info: Arc<LocationInfo>,
}

impl PluginContentProviderProxy {
    pub fn new(
        plugin_id: String,
        instance: PluginHandle,
        capability: PluginContentProviderCapability,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            plugin_id,
            instance,
            capability,
            location_info,
        }
    }

    /// The handler of an operation a plugin may leave out. Without one the operation is
    /// unsupported, exactly as for a built-in provider that keeps the trait's default.
    fn handler<'a>(
        &self,
        handler: Option<&'a String>,
        operation: &'static str,
    ) -> Result<&'a str, InstanceError> {
        handler
            .map(String::as_str)
            .ok_or_else(|| self.unsupported(operation))
    }

    /// The error for an answer that decoded but cannot be accepted; a wrong shape already fails
    /// while decoding.
    fn invalid_answer(&self, handler_name: &str, reason: impl std::fmt::Display) -> InstanceError {
        tracing::error!(
            "Plugin '{}' returned an invalid answer from '{}': {}",
            self.plugin_id,
            handler_name,
            reason
        );

        InstanceError::ContentProviderError {
            reason: format!(
                "Plugin '{}' returned an invalid answer from '{handler_name}': {reason}",
                self.plugin_id
            ),
        }
    }

    fn install_plan_from_dto(
        &self,
        handler_name: &str,
        requested: &ResolveTarget,
        dto: InstallPlanDto,
    ) -> Result<InstallPlan, InstanceError> {
        let mut target = InstallTarget::from(dto.target);
        if !requested.is_answered_by(&target) {
            return Err(self.invalid_answer(
                handler_name,
                format!(
                    "the plan is for {}, but {} was asked for",
                    describe_install_target(&target),
                    describe_resolve_target(requested)
                ),
            ));
        }

        // Held to the plugin's cache like the overrides: a host path would let the plugin show
        // any file of the user's as the instance icon.
        if let InstallTarget::New(instance) = &mut target
            && let Some(icon_path) = instance.icon_path.take()
        {
            let host_path = resolve_plugin_path(
                &self.location_info,
                &self.plugin_id,
                &icon_path,
                MountSet::CACHE,
            )
            .map_err(|err| self.invalid_answer(handler_name, err))?;
            instance.icon_path = Some(host_path.to_string_lossy().into_owned());
        }

        let files = dto
            .files
            .into_iter()
            .map(InstallFileEntry::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| self.invalid_answer(handler_name, err))?;

        let overrides = dto
            .overrides
            .into_iter()
            .map(|overrides| self.override_source_from_dto(overrides))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| self.invalid_answer(handler_name, err))?;

        Ok(InstallPlan {
            target,
            files,
            overrides,
        })
    }

    /// Override paths name files in the plugin's own `/cache` and nothing else (see
    /// [`resolve_plugin_path`]).
    fn override_source_from_dto(
        &self,
        dto: OverrideSourceDto,
    ) -> Result<OverrideSource, PluginError> {
        let to_host = |path: &str| {
            resolve_plugin_path(&self.location_info, &self.plugin_id, path, MountSet::CACHE)
        };

        let location = match dto.location {
            OverrideLocationDto::Archive {
                archive_path,
                subdir,
            } => OverrideLocation::Archive {
                archive_path: to_host(&archive_path)?,
                subdir,
            },
            OverrideLocationDto::Directory { path } => OverrideLocation::Directory {
                path: to_host(&path)?,
            },
        };

        Ok(OverrideSource::new(location, dto.side.into()))
    }

    /// A pack file is handed over under the plugin's own name for it; a file the plugin cannot
    /// read (outside `/cache` and `/instances`) is refused instead of passed on as a path that
    /// would fail inside the plugin.
    fn pack_source_to_dto(&self, source: PackSource) -> Result<PackSourceDto, InstanceError> {
        Ok(match source {
            PackSource::Content {
                content_id,
                version,
                installed_version,
            } => PackSourceDto::Content {
                content_id,
                version,
                installed_version,
            },
            PackSource::Url(url) => PackSourceDto::Url(url),
            PackSource::File(path) => PackSourceDto::File(
                host_path_to_plugin(&self.location_info, &self.plugin_id, &path).ok_or_else(
                    || InstanceError::ContentProviderError {
                        reason: format!(
                            "Plugin '{}' cannot read {}: a pack file has to be in the plugin's \
                             cache or in an instance",
                            self.plugin_id,
                            path.display()
                        ),
                    },
                )?,
            ),
        })
    }

    async fn call_plugin<I, O>(
        &self,
        operation: &'static str,
        handler_name: &str,
        input: I,
    ) -> Result<O, InstanceError>
    where
        I: Serialize + Send + 'static,
        O: DeserializeOwned + Send + 'static,
    {
        self.call_plugin_with(operation, handler_name, input, false)
            .await
    }

    /// `deploys_pack`: the call creates and fills an instance for a pack, which stays
    /// `PackInstalling` until the call is over (`infra::pack_instances`).
    async fn call_plugin_with<I, O>(
        &self,
        operation: &'static str,
        handler_name: &str,
        input: I,
        deploys_pack: bool,
    ) -> Result<O, InstanceError>
    where
        I: Serialize + Send + 'static,
        O: DeserializeOwned + Send + 'static,
    {
        let handler = handler_name.to_owned();
        let plugin_id = self.plugin_id.clone();
        let result = self
            .instance
            .run(handler_name, move |plugin| {
                plugin.supports(&handler).then(|| {
                    let call = || {
                        plugin
                            .call::<Msgpack<I>, Msgpack<O>>(&handler, Msgpack(input))
                            .map(|res| res.0)
                    };
                    if deploys_pack {
                        deploying_packs(&plugin_id, call).0
                    } else {
                        call()
                    }
                })
            })
            .await
            .and_then(Option::transpose);

        match result {
            Ok(Some(output)) => Ok(output),
            Ok(None) => {
                tracing::error!(
                    "Plugin '{}' missing handler '{}' for capability '{}'",
                    self.plugin_id,
                    handler_name,
                    self.capability.id
                );
                Err(self.unsupported(operation))
            }
            Err(PluginError::Stopped { plugin_id }) => {
                Err(InstanceError::PluginNotRunning { plugin_id })
            }
            Err(err) => {
                tracing::error!(
                    "Error calling plugin '{}' ({}): {:?}",
                    self.plugin_id,
                    handler_name,
                    err
                );
                Err(InstanceError::ContentProviderError {
                    reason: err.to_string(),
                })
            }
        }
    }
}

#[async_trait]
impl ContentProvider for PluginContentProviderProxy {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata {
        &self.capability
    }

    fn provider_id(&self) -> ProviderId {
        ProviderId {
            plugin_id: self.plugin_id.clone(),
            capability_id: self.capability.metadata.base.id.clone(),
        }
    }

    async fn search(
        &self,
        search_content: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        let dto: ContentSearchParamsDto = search_content.into();
        let result_dto: ContentSearchResultDto = self
            .call_plugin("search", &self.capability.handlers.search, dto)
            .await?;
        // Convert back from DTO — build domain result manually
        Ok(ContentSearchResult {
            page: result_dto.page,
            page_size: result_dto.page_size,
            page_count: result_dto.page_count,
            provider_id: result_dto.provider_id.into(),
            items: result_dto
                .items
                .into_iter()
                .map(|item| ContentItem {
                    id: item.id,
                    slug: item.slug,
                    name: item.name,
                    description: item.description,
                    long_description: item.long_description,
                    author: item.author,
                    url: item.url,
                    icon_url: item.icon_url,
                    versions: item.versions,
                    content_type: item.content_type.into(),
                })
                .collect(),
        })
    }

    async fn get_content(&self, content_id: String) -> Result<ContentItem, InstanceError> {
        let result_dto: aether_core_plugin_api::v0::ContentItemDto = self
            .call_plugin(
                "get_content",
                &self.capability.handlers.get_content,
                content_id,
            )
            .await?;
        Ok(result_dto.into())
    }

    async fn list_versions(
        &self,
        content_id: String,
    ) -> Result<Vec<ContentVersion>, InstanceError> {
        let handler = self.handler(
            self.capability.handlers.list_version.as_ref(),
            "list_versions",
        )?;
        let result_dto: Vec<ContentVersionDto> = self
            .call_plugin("list_versions", handler, content_id)
            .await?;
        Ok(result_dto.into_iter().map(Into::into).collect())
    }

    async fn resolve_file(
        &self,
        content_id: &str,
        version: Option<&str>,
        game_version: &str,
        loader: Option<&str>,
    ) -> Result<ResolvedFile, InstanceError> {
        let handler = self.handler(
            self.capability.handlers.resolve_file.as_ref(),
            "resolve_file",
        )?;
        let params = ResolveFileParamsDto {
            content_id: content_id.to_owned(),
            version: version.map(ToOwned::to_owned),
            game_version: game_version.to_owned(),
            loader: loader.map(ToOwned::to_owned),
        };

        let result_dto: ResolvedFileDto = self.call_plugin("resolve_file", handler, params).await?;
        result_dto
            .try_into()
            .map_err(|err| self.invalid_answer(handler, err))
    }

    async fn resolve_install_plan(
        &self,
        params: ResolveInstallPlanParams,
    ) -> Result<InstallPlan, InstanceError> {
        let handler = self.handler(
            self.capability.handlers.resolve_install_plan.as_ref(),
            "resolve_install_plan",
        )?;
        let requested = params.target.clone();

        let result_dto: InstallPlanDto = self
            .call_plugin(
                "resolve_install_plan",
                handler,
                ResolveInstallPlanParamsDto::from(params),
            )
            .await?;
        self.install_plan_from_dto(handler, &requested, result_dto)
    }

    async fn resolve_pack_metadata(
        &self,
        source: PackSource,
    ) -> Result<PackMetadata, InstanceError> {
        let handler = self.handler(
            self.capability.handlers.resolve_pack_metadata.as_ref(),
            "resolve_pack_metadata",
        )?;
        let source = self.pack_source_to_dto(source)?;

        let result_dto: PackMetadataDto = self
            .call_plugin("resolve_pack_metadata", handler, source)
            .await?;
        Ok(result_dto.into())
    }

    async fn check_compatibility(
        &self,
        instances: &[Instance],
        check_params: &ContentCompatibilityCheckParams,
    ) -> Result<HashMap<String, ContentCompatibilityResult>, InstanceError> {
        let handler = self.handler(
            self.capability.handlers.check_compatibility.as_ref(),
            "check_compatibility",
        )?;
        let params: PluginCheckCompatibilityParamsDto = PluginCheckCompatibilityParams {
            instances: instances.to_vec(),
            check_params: check_params.clone(),
        }
        .into();

        let result_dto: HashMap<String, aether_core_plugin_api::v0::ContentCompatibilityResultDto> =
            self.call_plugin("check_compatibility", handler, params)
                .await?;
        Ok(result_dto
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    ContentCompatibilityResult {
                        is_compatible: v.is_compatible,
                    },
                )
            })
            .collect())
    }
}

fn describe_install_target(target: &InstallTarget) -> String {
    match target {
        InstallTarget::Existing { instance_id, .. } => format!("instance '{instance_id}'"),
        InstallTarget::New(_) => "a new instance".to_owned(),
    }
}

fn describe_resolve_target(target: &ResolveTarget) -> String {
    match target {
        ResolveTarget::Existing { instance_id, .. } => format!("instance '{instance_id}'"),
        ResolveTarget::New => "a new instance".to_owned(),
    }
}
