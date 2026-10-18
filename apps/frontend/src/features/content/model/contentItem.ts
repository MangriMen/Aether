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
  ModpackInstallParamsDto,
  ProviderIdDto,
} from '@/platform/bindings/instance';

import type { ContentType } from './contentType';

export type ProviderId = ProviderIdDto;

export type ContentItem = ContentItemDto;

export interface ContentSearchParams extends Omit<
  ContentSearchParamsDto,
  'page' | 'pageSize'
> {
  page: number;
  pageSize: number;
}

export interface ContentSearchResponse extends Omit<
  ContentSearchResultDto,
  'page' | 'pageSize' | 'pageCount'
> {
  page: number;
  pageSize: number;
  pageCount: number;
}

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type AtomicInstallParams = AtomicInstallParamsDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type ModpackInstallParams = ModpackInstallParamsDto;

export type InstallContentParams = ContentInstallParamsDto;

export interface ContentFilters {
  page?: number;
  pageSize?: number;
  query?: string;
  providerId?: ProviderId;
  contentType?: ContentType;
  gameVersions?: string[];
  loaders?: string[];
}

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type ContentVersionDependencyType = ContentVersionDependencyTypeDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type ContentVersionDependency = ContentVersionDependencyDto;

/** @public Dto alias, kept while unused (ARCHITECTURE.md, Checks). */
export type ContentVersionType = ContentVersionTypeDto;

export interface ContentVersion extends Omit<ContentVersionDto, 'downloads'> {
  downloads: number;
}

export type ContentGetParams = ContentGetParamsDto;

export type ContentListVersionParams = ContentListVersionParamsDto;
