import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { settingsCommands } from '../../api';
import { appSettingsCache, appSettingsQueries } from './cache';

export const useAppSettings = () => useQuery(appSettingsQueries.get);

export const useEditAppSettings = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: settingsCommands.editAppSettings,
    onSuccess: () => appSettingsCache.invalidate.get(queryClient),
  }));
};
