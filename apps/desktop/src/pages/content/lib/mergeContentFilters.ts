import { mergeDefined } from '@/shared/lib';

/**
 * High-level merge for content state.
 */
export function mergeContentFilters<T extends object, L extends object>(
  ...sources: Array<{ filters?: T; filtersLock?: L }>
) {
  const nextFilters = mergeDefined<T>(...sources.map((s) => s.filters));
  const nextLock = mergeDefined<L>(...sources.map((s) => s.filtersLock));

  //  Ensure we return undefined if result objects are empty
  const hasFilters = Object.keys(nextFilters).length > 0;
  const hasLock = Object.keys(nextLock).length > 0;

  return {
    filters: hasFilters ? nextFilters : undefined,
    filtersLock: hasLock ? nextLock : undefined,
  };
}
