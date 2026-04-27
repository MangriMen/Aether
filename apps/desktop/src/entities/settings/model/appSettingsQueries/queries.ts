import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { commands } from '../../api';
import { appSettingsCache, appSettingsQueries } from './cache';

export const useAppSettings = () => useQuery(appSettingsQueries.get);

export const useEditAppSettings = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: commands.editAppSettings,
    onSuccess: () => appSettingsCache.invalidate.get(queryClient),
  }));
};

export const useRecreateWindow = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: commands.recreateWindow,
    onSuccess: () => appSettingsCache.invalidate.get(queryClient),
  }));
};
