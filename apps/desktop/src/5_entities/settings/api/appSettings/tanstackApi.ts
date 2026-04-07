import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { APP_SETTINGS_QUERY_KEYS } from './appSettingsQueryKeys';
import { getAppSettingsRaw, editAppSettingsRaw } from './tauriApi';

export const useAppSettings = () =>
  useQuery(() => ({
    queryKey: APP_SETTINGS_QUERY_KEYS.GET(),
    queryFn: getAppSettingsRaw,
  }));

export const useEditAppSettings = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: editAppSettingsRaw,
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: APP_SETTINGS_QUERY_KEYS.GET(),
      });
    },
  }));
};
