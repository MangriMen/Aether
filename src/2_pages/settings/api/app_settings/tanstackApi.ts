import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import { APP_SETTINGS_QUERY_KEYS } from './appSettingsQueryKeys';
import { getAppSettingsRaw, updateAppSettingsRaw } from './rawApi';

export const useAppSettings = () =>
  useQuery(() => ({
    queryKey: APP_SETTINGS_QUERY_KEYS.GET(),
    queryFn: getAppSettingsRaw,
  }));

export const useUpdateAppSettings = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: updateAppSettingsRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: APP_SETTINGS_QUERY_KEYS.GET(),
      });
    },
  }));
};
