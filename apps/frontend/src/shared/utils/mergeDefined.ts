/**
 * Merges objects, skipping undefined values.
 * Later sources overwrite earlier ones.
 */
export function mergeDefined<T extends object>(
  ...sources: Array<T | undefined>
): T {
  // Use reduce with explicit initial value type to satisfy TS generic constraints
  return sources.reduce<T>((acc, source) => {
    //  Skip null, undefined or non-object sources
    if (!source || typeof source !== 'object' || Array.isArray(source)) {
      return acc;
    }

    const cleanEntries = Object.entries(source).filter(
      ([_, v]) => v !== undefined,
    );

    return {
      ...acc,
      ...Object.fromEntries(cleanEntries),
    } as T;
  }, {} as T);
}
