import type {
  CacheUsageDto,
  EditSettingsDto,
  SettingsDto,
  StorageKindDto,
  StorageUsageDto,
} from '@/platform/bindings/settings';

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type Settings = SettingsDto;
/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type EditSettings = EditSettingsDto;

export type CacheUsage = CacheUsageDto;

export type StorageKind = StorageKindDto;

export type StorageUsage = StorageUsageDto;
