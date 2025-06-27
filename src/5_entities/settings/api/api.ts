import type { QueryClient } from '@tanstack/solid-query';
import { useQuery } from '@tanstack/solid-query';
import { QUERY_KEYS } from './queryKeys';
import { getMaxRamRaw, getSettingsRaw } from './rawApi';

export const useSettings = () =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.SETTINGS.GET(),
    queryFn: getSettingsRaw,
  }));

export const MAX_RAM_QUERY = () => ({
  queryKey: QUERY_KEYS.SETTINGS.RAM(),
  queryFn: getMaxRamRaw,
  staleTime: Infinity,
});

export const useMaxRam = () => useQuery(MAX_RAM_QUERY);

export const prefetchMaxRam = (queryClient: QueryClient) =>
  queryClient.prefetchQuery(MAX_RAM_QUERY());
