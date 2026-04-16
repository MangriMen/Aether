import type {
  ModdedLoaderVersionDto,
  ModdedManifestDto,
  ModdedVersionDto,
} from '@/shared/api/bindings/minecraft';

export type ModdedManifest = ModdedManifestDto;

export type ModdedVersion = ModdedVersionDto;

export type ModdedLoaderVersion = ModdedLoaderVersionDto;

export interface MappedLoaderManifest {
  gameVersions: Record<string, ModdedVersion>;
}
