const GLOB_CHARS = /[*?[\]]/;

/**
 * An `allowed_hosts` entry that admits far more than one site: a glob with fewer than two
 * literal labels left (`*`, `*.*`, `*.com`). `*.example.com` is not wide: it stays on one domain.
 */
export const isWideHostPattern = (entry: string): boolean => {
  const host = entry.trim();

  if (!GLOB_CHARS.test(host)) {
    return false;
  }

  const literalLabels = host
    .split('.')
    .filter((label) => label.length > 0 && !GLOB_CHARS.test(label));

  return literalLabels.length < 2;
};
