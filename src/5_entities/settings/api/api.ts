import type { QueryClient } from '@tanstack/solid-query';

import { useQuery } from '@tanstack/solid-query';

import { QUERY_KEYS } from './queryKeys';
import { getMaxRamRaw, getSettingsRaw } from './rawApi';

export const useSettings = () =>
  useQuery(() => ({
    queryFn: getSettingsRaw,
    queryKey: QUERY_KEYS.SETTINGS.GET(),
  }));

export const MAX_RAM_QUERY = () => ({
  queryFn: getMaxRamRaw,
  queryKey: QUERY_KEYS.SETTINGS.RAM(),
  staleTime: Infinity,
});

export const useMaxRam = () => useQuery(MAX_RAM_QUERY);

export const prefetchMaxRam = (queryClient: QueryClient) =>
  queryClient.prefetchQuery(MAX_RAM_QUERY());
