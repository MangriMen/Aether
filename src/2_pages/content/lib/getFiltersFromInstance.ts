import type { ContentFilters, Instance } from '@/entities/instances';

import type { ContentFiltersLock } from '../model/contentFiltersLock';

export const getFiltersFromInstance = (
  instance?: Instance,
): { filters?: ContentFilters; filtersLock?: ContentFiltersLock } => {
  if (!instance) {
    return { filters: undefined, filtersLock: undefined };
  }

  const instanceGameVersion = instance?.gameVersion;
  const instanceLoader = instance?.loader;

  return {
    filters: {
      gameVersions: instanceGameVersion ? [instanceGameVersion] : undefined,
      loaders: instanceLoader ? [instanceLoader] : undefined,
    },
    filtersLock: {
      gameVersion: !!instanceGameVersion,
      loader: !!instanceLoader,
    },
  };
};
