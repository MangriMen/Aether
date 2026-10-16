const UNITS = ['byte', 'kilobyte', 'megabyte', 'gigabyte', 'terabyte'] as const;

/**
 * A size in bytes as people read it, in the language of the interface:
 * `0 B`, `1.5 MB` — `1,5 МБ` in Russian. Steps of 1024, as file managers do.
 */
export const formatBytes = (bytes: number, locale?: string): string => {
  let value = Math.max(0, bytes);
  let unit = 0;

  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }

  return new Intl.NumberFormat(locale, {
    style: 'unit',
    unit: UNITS[unit],
    unitDisplay: 'short',
    maximumFractionDigits: unit === 0 || value >= 100 ? 0 : 1,
  }).format(value);
};

/** The total of sizes that may be missing: specta exports `f64` as `number | null`. */
export const sumBytes = (entries: { bytes: number | null }[]): number =>
  entries.reduce((sum, entry) => sum + (entry.bytes ?? 0), 0);
