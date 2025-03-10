import type { ContentType } from './contentType';

export interface ContentItem {
  id: string;
  name: string;
  description?: string;
  author: string;
  contentType: ContentType;
  url: string;
}

export interface ContentRequest {
  contentType: ContentType;
  provider: string;
  page: number;
  pageSize: number;
  query?: string;
}

export interface ContentResponse {
  page: number;
  pageSize: number;
  pageCount: number;
  provider: string;
  items: ContentItem[];
}
