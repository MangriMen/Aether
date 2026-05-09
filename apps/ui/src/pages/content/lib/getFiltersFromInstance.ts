import type { ContentFilters, Instance } from '@/entities/instances';

import type { ContentFiltersLock } from '../model';

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
