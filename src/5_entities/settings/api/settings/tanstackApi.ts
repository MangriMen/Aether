import type { QueryClient } from '@tanstack/solid-query';
import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import { editSettingsRaw, getMaxRamRaw, getSettingsRaw } from './tauriApi';
import { showError } from '@/shared/lib/showError';
import { useTranslation } from '@/shared/model';
import { SETTINGS_QUERY_KEYS } from './settingsQueryKeys';

export const useSettings = () =>
  useQuery(() => ({
    queryKey: SETTINGS_QUERY_KEYS.GET(),
    queryFn: getSettingsRaw,
  }));

export const useEditSettings = () => {
  const queryClient = useQueryClient();

  const [{ t }] = useTranslation();

  return useMutation(() => ({
    mutationFn: editSettingsRaw,
    onSuccess: (data) => {
      queryClient.setQueryData(SETTINGS_QUERY_KEYS.GET(), () => data);
    },
    onError: (err) => {
      showError({
        title: t('settings.changeError'),
        err,
        t,
      });
    },
  }));
};

export const MAX_RAM_QUERY = () => ({
  queryKey: SETTINGS_QUERY_KEYS.RAM(),
  queryFn: getMaxRamRaw,
  staleTime: Infinity,
});

export const useMaxRam = () => useQuery(MAX_RAM_QUERY);

export const prefetchMaxRam = (queryClient: QueryClient) =>
  queryClient.prefetchQuery(MAX_RAM_QUERY());
