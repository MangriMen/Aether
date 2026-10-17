import type { ContentFilters, ContentFiltersLock } from '@/features/content';
import type { Instance } from '@/features/instance';

export const getFiltersFromInstance = (
  instance?: Instance,
): { filters?: ContentFilters; filtersLock?: ContentFiltersLock } => {
  if (!instance) {
    return { filters: undefined, filtersLock: undefined };
  }

  return {
    filters: {
      gameVersions: [instance.gameVersion],
      loaders: [instance.loader],
    },
    filtersLock: {
      gameVersion: true,
      loader: true,
    },
  };
};
