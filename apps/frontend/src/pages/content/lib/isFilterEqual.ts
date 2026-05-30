import type { ContentFilters } from '@/entities/instances';

export const isFiltersEqual = (
  a: Partial<ContentFilters> | undefined,
  b: Partial<ContentFilters> | undefined,
): boolean => {
  if (a === b) return true;
  if (!a || !b) return false;

  const keys = new Set([...Object.keys(a), ...Object.keys(b)]);

  for (const key of keys) {
    const valA = a[key as keyof ContentFilters];
    const valB = b[key as keyof ContentFilters];

    // Specialized check for nested providerId object
    if (key === 'providerId') {
      const pA = valA as ContentFilters['providerId'];
      const pB = valB as ContentFilters['providerId'];
      if (
        pA?.pluginId !== pB?.pluginId ||
        pA?.capabilityId !== pB?.capabilityId
      ) {
        return false;
      }
      continue;
    }

    // Standard shallow comparison for other fields
    if (valA !== valB) return false;
  }

  return true;
};
