import type { SearchParams } from '@solidjs/router';

export const searchParamsToQueryString = (params: SearchParams): string => {
  const search = new URLSearchParams();

  Object.entries(params).forEach(([key, value]) => {
    if (value === undefined || value === null) return;

    if (Array.isArray(value)) {
      value.forEach((v) => search.append(key, v));
    } else {
      search.set(key, String(value));
    }
  });

  return search.toString();
};
