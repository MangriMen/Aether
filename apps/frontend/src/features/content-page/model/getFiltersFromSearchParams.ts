import type {
  ContentFilters,
  ContentFiltersLock,
  ContentPageSearchParams,
} from '@/features/content';

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
      providerId: searchParams.providerId,
      contentType: searchParams.contentType,
      gameVersions: searchParams.gameVersions,
      loaders: searchParams.loaders,
    },
    filtersLock: undefined,
  };
};
