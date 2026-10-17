import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';

import { settingsCommands } from '@/platform/ipc';

import { appSettingsCache, appSettingsQueries } from './appSettingsCache';

export const useAppSettings = () => useQuery(appSettingsQueries.get);

export const useEditAppSettings = () => {
  const queryClient = useQueryClient();

  return useMutation(() => ({
    mutationFn: settingsCommands.editAppSettings,
    onSuccess: () => appSettingsCache.invalidate.get(queryClient),
  }));
};
