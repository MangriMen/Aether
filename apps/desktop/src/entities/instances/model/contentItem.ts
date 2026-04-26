import type { ContentSearchOutputValues } from '@/pages/content/model/validation';

import type {
  AtomicInstallParamsDto,
  ContentGetParamsDto,
  ContentInstallParamsDto,
  ContentItemDto,
  ContentListVersionParamsDto,
  ContentSearchParamsDto,
  ContentSearchResultDto,
  ContentVersionDependencyDto,
  ContentVersionDependencyTypeDto,
  ContentVersionDto,
  ContentVersionTypeDto,
  ModLoaderDto,
  ModpackInstallParamsDto,
  ProviderIdDto,
} from '../api';

export type ProviderId = ProviderIdDto;

export type ContentItem = ContentItemDto;

export type ContentSearchParams = ContentSearchParamsDto;

export type ContentSearchResponse = ContentSearchResultDto;

export type AtomicInstallParams = AtomicInstallParamsDto;

export type ModpackInstallParams = ModpackInstallParamsDto;

export type InstallContentParams = ContentInstallParamsDto;

export type ContentFilters = Partial<ContentSearchOutputValues>;

export const PROVIDER_ID_SEPARATOR = '#' as const;

export type ProviderIdString =
  `${string}${typeof PROVIDER_ID_SEPARATOR}${string}`;

export const providerIdToString = (
  providerId: ProviderId,
): ProviderIdString => {
  return `${providerId.pluginId}${PROVIDER_ID_SEPARATOR}${providerId.capabilityId}`;
};

export type ContentVersionDependencyType = ContentVersionDependencyTypeDto;

export type ContentVersionDependency = ContentVersionDependencyDto;

export type ContentVersionType = ContentVersionTypeDto;

export type ContentVersion = ContentVersionDto;

export type ContentGetParams = ContentGetParamsDto;

export type ContentListVersionParams = ContentListVersionParamsDto;

export type ModLoader = ModLoaderDto;

export const MOD_LOADERS: ModLoader[] = [
  'vanilla',
  'forge',
  'fabric',
  'quilt',
  'neoforge',
];

//@ts-expect-error Used for strict type checking
const _TYPE_CHECK: Record<ModLoader, boolean> = {
  vanilla: true,
  forge: true,
  fabric: true,
  quilt: true,
  neoforge: true,
};

export function isModLoader(value: unknown): value is ModLoader {
  return typeof value === 'string' && MOD_LOADERS.includes(value as ModLoader);
}
