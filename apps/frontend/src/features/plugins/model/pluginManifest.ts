import type {
  ApiConfigDto,
  LoadConfigDto,
  PathMappingDto,
  PluginManifestDto,
  PluginMetadataDto,
  RuntimeConfigDto,
} from '@/platform/bindings/plugin';

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type PluginManifest = PluginManifestDto;
export type PluginMetadata = PluginMetadataDto;

export type RuntimeConfig = RuntimeConfigDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type PathMapping = PathMappingDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type LoadConfig = LoadConfigDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type ApiConfig = ApiConfigDto;

export type PluginId = PluginMetadataDto['id'];
