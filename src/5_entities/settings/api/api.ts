import { useQuery } from '@tanstack/solid-query';
import { QUERY_KEYS } from './queryKeys';
import { getMaxRamRaw, getSettingsRaw } from './rawApi';

export const useSettings = () =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.SETTINGS.GET(),
    queryFn: getSettingsRaw,
  }));

export const useMaxRam = () =>
  useQuery(() => ({
    queryKey: QUERY_KEYS.SETTINGS.RAM(),
    queryFn: getMaxRamRaw,
  }));
