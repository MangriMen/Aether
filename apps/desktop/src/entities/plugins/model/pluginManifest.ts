import type {
  ApiConfigDto,
  LoadConfigDto,
  PathMappingDto,
  PluginManifestDto,
  PluginMetadataDto,
  RuntimeConfigDto,
} from '../api';

export type PluginManifest = PluginManifestDto;
export type PluginMetadata = PluginMetadataDto;

export type RuntimeConfig = RuntimeConfigDto;

export type PathMapping = PathMappingDto;

export type LoadConfig = LoadConfigDto;

export type ApiConfig = ApiConfigDto;

export type PluginId = PluginMetadataDto['id'];
