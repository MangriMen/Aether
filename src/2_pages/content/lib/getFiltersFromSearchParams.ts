import type { ContentFilters } from '@/entities/instances';

import type { ContentFiltersLock } from '../model/contentFiltersLock';
import type { ContentPageSearchParams } from '../model/contentPageSearchParams';

export const getFiltersFromSearchParams = (
  searchParams: ContentPageSearchParams,
): {
  filters?: ContentFilters;
  filtersLock?: ContentFiltersLock;
} => {
  return {
    filters: {
      page: searchParams.page,
      pageSize: searchParams.pageSize,
      query: searchParams.query,
      provider: searchParams.provider,
      contentType: searchParams.contentType,
      gameVersions: searchParams.gameVersions,
      loaders: searchParams.loaders,
    },
    filtersLock: undefined,
  };
};
