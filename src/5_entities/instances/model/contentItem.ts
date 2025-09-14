import type { ContentType } from './contentType';

export interface ContentItem {
  author: string;
  contentType: ContentType;
  description?: string;
  iconUrl: string;
  id: string;
  name: string;
  providerData?: Record<string, unknown>;
  url: string;
  versions: string[];
}

export interface ContentItemExtended extends ContentItem {
  installed: boolean;
}

export interface ContentRequest {
  contentType: ContentType;
  gameVersions?: string[];
  loader?: string;
  page: number;
  pageSize: number;
  provider: string;
  query?: string;
}

export interface ContentResponse {
  items: ContentItem[];
  page: number;
  pageCount: number;
  pageSize: number;
  provider: string;
}

export interface InstallContentPayload {
  contentType: ContentType;
  contentVersion?: string;
  gameVersion: string;
  loader?: string;
  provider: string;
  providerData?: unknown;
}
