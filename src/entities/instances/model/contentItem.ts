import type { ContentType } from './contentType';

export interface ContentItem {
  id: string;
  name: string;
  description?: string;
  author: string;
  url: string;
  iconUrl: string;
  versions: string[];
  contentType: ContentType;
  providerData?: unknown;
}

export interface ContentRequest {
  contentType: ContentType;
  provider: string;
  page: number;
  pageSize: number;
  query?: string;
  gameVersions?: string[];
  loader?: string;
}

export interface ContentResponse {
  page: number;
  pageSize: number;
  pageCount: number;
  provider: string;
  items: ContentItem[];
}

export interface InstallContentPayload {
  gameVersion: string;
  loader?: string;
  contentType: ContentType;
  contentVersion?: string;
  provider: string;
  providerData?: unknown;
}
