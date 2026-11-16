use register_schema::RegisterSchema;
use schemars::JsonSchema;

use super::{
    CachedFileDto, CommandDto, ContentAuthorDto, ContentCompatibilityCheckParamsDto,
    ContentCompatibilityResultDto, ContentFileDto, ContentItemDto, ContentMetadataDto,
    ContentSearchParamsDto, ContentSearchResultDto, ContentVersionDto, ContentVersionMetadataDto,
    DownloadToCacheParamsDto, GetAuthorsParamsDto, GetContentsMetadataParamsDto,
    GetVersionsMetadataParamsDto, HostError, HttpPostJsonRequestDto, HttpRequestDto,
    HttpResponseDto, InstallPlanDto, InstanceFileBytesDto, JavaDto, NewInstanceDto, OutputDto,
    PackMetadataDto, PackSourceDto, PluginCheckCompatibilityParamsDto, PluginImportInstanceDto,
    PluginInternalEventDto, ReadInstanceFileParamsDto, ResolveFileParamsDto,
    ResolveInstallPlanParamsDto, ResolvedFileDto, WriteInstanceFileParamsDto,
};

/// Every structured value that crosses the plugin boundary, in one schema: the properties only
/// name the types, which are described under `$defs`. Which one a host function or an export
/// takes and returns is in the README of the plugin API; how a value is encoded is in its
/// "Wire format" section.
// Published as `schemas/plugin-wire.schema.json`. A plain `//` comment, so that it stays out of
// the schema `description`; see `PluginManifestDto`.
#[derive(JsonSchema, RegisterSchema)]
#[schemars(rename = "PluginWire", rename_all = "camelCase")]
#[schema_category("plugin_api")]
#[schema_name("PluginWire")]
#[allow(dead_code)]
pub struct PluginWireDto {
    /// What every host function returns on failure, inside `{"status": "err", "data": …}`.
    pub host_error: HostError,

    // Host functions: what the plugin sends.
    pub command: CommandDto,
    pub new_instance: NewInstanceDto,
    pub write_instance_file: WriteInstanceFileParamsDto,
    pub read_instance_file: ReadInstanceFileParamsDto,
    pub http_request: HttpRequestDto,
    pub http_post_json_request: HttpPostJsonRequestDto,
    pub download_to_cache: DownloadToCacheParamsDto,

    // Host functions: what comes back inside `{"status": "ok", "data": …}`.
    pub output: OutputDto,
    pub java: JavaDto,
    pub content_files: std::collections::HashMap<String, ContentFileDto>,
    pub instance_file_bytes: InstanceFileBytesDto,
    pub http_response: HttpResponseDto,
    pub cached_file: CachedFileDto,

    // Exports the host calls: what it sends.
    pub internal_event: PluginInternalEventDto,
    pub import_instance: PluginImportInstanceDto,
    pub content_search_params: ContentSearchParamsDto,
    pub resolve_file_params: ResolveFileParamsDto,
    pub resolve_install_plan_params: ResolveInstallPlanParamsDto,
    pub pack_source: PackSourceDto,
    pub get_contents_metadata_params: GetContentsMetadataParamsDto,
    pub get_versions_metadata_params: GetVersionsMetadataParamsDto,
    pub get_authors_params: GetAuthorsParamsDto,
    pub check_compatibility_params: PluginCheckCompatibilityParamsDto,
    pub content_compatibility_check_params: ContentCompatibilityCheckParamsDto,

    // Exports the host calls: what they answer.
    pub content_search_result: ContentSearchResultDto,
    pub content_item: ContentItemDto,
    pub content_versions: Vec<ContentVersionDto>,
    pub resolved_file: ResolvedFileDto,
    pub install_plan: InstallPlanDto,
    pub pack_metadata: PackMetadataDto,
    pub contents_metadata: Vec<ContentMetadataDto>,
    pub versions_metadata: Vec<ContentVersionMetadataDto>,
    pub authors: Vec<ContentAuthorDto>,
    pub compatibility_result: std::collections::HashMap<String, ContentCompatibilityResultDto>,
}
