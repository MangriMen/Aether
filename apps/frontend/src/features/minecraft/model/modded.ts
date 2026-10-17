import type {
  ModdedLoaderVersionDto,
  ModdedManifestDto,
  ModdedVersionDto,
  ModdedVersionGroupDto,
} from '@/platform/bindings/minecraft';

export type ModdedManifest = ModdedManifestDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type ModdedVersion = ModdedVersionDto;

export type ModdedLoaderVersion = ModdedLoaderVersionDto;

export interface MappedLoaderManifest {
  gameVersions: Record<string, ModdedVersion>;
  versionGroups: Record<string, ModdedVersionGroupDto>;
}
