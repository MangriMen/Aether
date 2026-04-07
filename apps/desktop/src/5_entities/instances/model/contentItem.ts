import type { ModLoader } from '@/entities/minecraft';
import type { ContentSearchOutputValues } from '@/pages/content/model/validation';

import type { AtomicContentType, ContentType } from './contentType';

export interface ProviderId {
  pluginId: string;
  capabilityId: string;
}

export interface ContentItem {
  id: string;
  slug: string;
  name: string;
  description?: string;
  longDescription?: string;
  author: string;
  url: string;
  iconUrl: string;
  versions: string[];
  contentType: ContentType;
}

export interface ContentSearchParams {
  contentType: ContentType;
  providerId: ProviderId;
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
  providerId: ProviderId;
  items: ContentItem[];
}

export interface AtomicInstallParams {
  instanceId: string;
  gameVersion: string;
  loader?: string;
  contentId: string;
  contentType: AtomicContentType;
  contentVersion?: string;
  providerId: ProviderId;
}

export interface ModpackInstallParams {
  contentId: string;
  contentVersion?: string;
  providerId: ProviderId;
}

export type InstallContentParams =
  | { type: 'atomic'; data: AtomicInstallParams }
  | { type: 'modpack'; data: ModpackInstallParams };

export type ContentFilters = Partial<ContentSearchOutputValues>;

export const PROVIDER_ID_SEPARATOR = '#' as const;

export type ProviderIdString =
  `${string}${typeof PROVIDER_ID_SEPARATOR}${string}`;

export const providerIdToString = (
  providerId: ProviderId,
): ProviderIdString => {
  return `${providerId.pluginId}${PROVIDER_ID_SEPARATOR}${providerId.capabilityId}`;
};

export type ContentVersionDependencyType =
  | 'required'
  | 'optional'
  | 'incompatible'
  | 'embedded';

export interface ContentVersionDependency {
  versionId?: string;
  contentId?: string;
  fileName?: string;
  dependencyType: ContentVersionDependencyType;
}

export type ContentVersionType = 'release' | 'beta' | 'alpha';

export interface ContentVersion {
  name: string;
  versionNumber: string;
  changelog?: string;
  dependencies: Array<ContentVersionDependency>;
  gameVersions: Array<string>;
  versionType: ContentVersionType;
  loaders: Array<ModLoader>;
  featured: boolean;
  status:
    | 'listed'
    | 'archived'
    | 'draft'
    | 'unlisted'
    | 'scheduled'
    | 'unknown';
  requestedStatus?: 'listed' | 'archived' | 'draft' | 'unlisted';
  id: string;
  contentId: string;
  authorId: string;
  datePublished: string;
  downloads: number;
  webUrl: string;
}

export interface ContentGetParams {
  contentId: string;
  providerId: ProviderId;
}

export interface ContentListVersionParams {
  contentId: string;
  providerId: ProviderId;
}
