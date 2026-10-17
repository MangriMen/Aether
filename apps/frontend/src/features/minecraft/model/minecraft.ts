import type {
  LatestVersionDto,
  VersionDto,
  VersionManifestDto,
  VersionTypeDto,
} from '@/platform/bindings/minecraft';

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type VersionType = VersionTypeDto;

export type Version = VersionDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type LatestVersion = LatestVersionDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type VersionManifest = VersionManifestDto;
