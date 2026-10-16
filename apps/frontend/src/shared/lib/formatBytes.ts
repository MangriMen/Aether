const UNITS = ['B', 'KB', 'MB', 'GB', 'TB'] as const;

/** A size in bytes as people read it: `0 B`, `1.5 MB`, `120 KB`. */
export const formatBytes = (bytes: number): string => {
  let value = Math.max(0, bytes);
  let unit = 0;

  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }

  const digits = unit === 0 || value >= 100 ? 0 : 1;
  return `${value.toFixed(digits)} ${UNITS[unit]}`;
};
