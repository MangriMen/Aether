import type { ContentSearchOutputValues } from '@/pages/content/model/validation';

import type { AtomicContentType, ContentType } from './contentType';

export interface ContentItem {
  id: string;
  slug: string;
  name: string;
  description?: string;
  author: string;
  url: string;
  iconUrl: string;
  versions: string[];
  contentType: ContentType;
}

export interface ContentSearchParams {
  contentType: ContentType;
  provider: string;
  page: number;
  pageSize: number;
  query?: string;
  gameVersions?: string[];
  loader?: string;
}

export interface ContentSearchResponse {
  page: number;
  pageSize: number;
  pageCount: number;
  provider: string;
  items: ContentItem[];
}

export interface AtomicInstallParams {
  instanceId: string;
  gameVersion: string;
  loader?: string;
  contentId: string;
  contentType: AtomicContentType;
  contentVersion?: string;
  provider: string;
}

export interface ModpackInstallParams {
  contentId: string;
  contentVersion?: string;
  provider: string;
}

export type InstallContentParams =
  | { type: 'atomic'; data: AtomicInstallParams }
  | { type: 'modpack'; data: ModpackInstallParams };

export type ContentFilters = Partial<ContentSearchOutputValues>;
