import type { ContentSearchInputValues } from './validation';

import { ContentType } from '../../../entities/instances';

export const DEFAULT_PER_PAGE_OPTIONS = [5, 10, 20, 30, 40, 50];

export const DEFAULT_FORM_VALUES = {
  page: 1,
  pageSize: 20,
  query: '',
  contentType: ContentType.Mod,
} as const satisfies Partial<ContentSearchInputValues>;

export const SEARCH_QUERY_DEBOUNCE_DELAY = 500;
