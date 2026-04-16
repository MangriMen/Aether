import type {
  LatestVersionDto,
  VersionDto,
  VersionManifestDto,
  VersionTypeDto,
} from '@/shared/api/bindings/minecraft';

export type VersionType = VersionTypeDto;

export type Version = VersionDto;

export type LatestVersion = LatestVersionDto;

export type VersionManifest = VersionManifestDto;

export const VERSION_TYPES: readonly VersionType[] = [
  'release',
  'snapshot',
  'old_alpha',
  'old_beta',
] as const;
