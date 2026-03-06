// English comments as requested.

/**
 * Merges multiple filter sources.
 * Sources provided later in the arguments list have higher priority.
 */
export function mergeContentFilters<T extends object, L extends object>(
  ...sources: Array<{ filters?: T; filtersLock?: L }>
) {
  return sources.reduce(
    (acc, current) => {
      // Helper to remove undefined keys so they don't overwrite previous values
      const clean = (obj?: object) =>
        obj
          ? Object.fromEntries(
              Object.entries(obj).filter(([_, v]) => v !== undefined),
            )
          : {};

      const nextFilters = { ...acc.filters, ...clean(current.filters) } as T;
      const nextLock = {
        ...acc.filtersLock,
        ...clean(current.filtersLock),
      } as L;

      const hasFilters = Object.keys(nextFilters).length > 0;
      const hasLock = Object.keys(nextLock).length > 0;

      return {
        filters: hasFilters ? nextFilters : undefined,
        filtersLock: hasLock ? nextLock : undefined,
      };
    },
    { filters: undefined, filtersLock: undefined } as {
      filters?: T;
      filtersLock?: L;
    },
  );
}
