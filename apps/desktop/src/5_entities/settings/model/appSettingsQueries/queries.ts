import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { commands } from '../../api';
import { appSettingsQueryKeys } from './queryKeys';

export const useAppSettings = () =>
  useQuery(() => ({
    queryKey: appSettingsQueryKeys.get(),
    queryFn: commands.getAppSettings,
  }));

export const useEditAppSettings = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: commands.editAppSettings,
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: appSettingsQueryKeys.get(),
      });
    },
  }));
};
